//! Letting go of tombstones: past their retention, then past the memory cap.

use std::{
    mem,
    time::{Duration, Instant},
};

use brk_types::Txid;

use super::{TxGraveyard, TxTombstone};

/// How long a tombstone outside any replacement tree stays.
const RETENTION: Duration = Duration::from_hours(1);
/// How long a replacement tree stays once its newest version left the pool (mempool.space's):
/// while that one is pending, the whole tree does.
const TREE_RETENTION: Duration = Duration::from_hours(24);
/// The memory the graveyard's bodies take at most ([`TxRecord::memory`]). Past it, the oldest lone
/// tombstones go first: they cost nothing to make (an eviction burst, or a node restarting with an
/// old mempool.dat, buries hundreds of thousands at once) and are kept an hour only. Then the oldest
/// replaced versions, each with what it replaced, then the oldest trees. Only a flood of
/// replacements pushes out older trees' histories, at a price: every replacement pays Core's
/// incremental relay fee over its size, about 0.2 BTC to fill it.
const MAX_MEMORY: u64 = 256 << 20;
/// The longest replacement chain an eviction check follows (the RBF listings' work bound).
const MAX_WALK: usize = 4_096;

impl TxGraveyard {
    /// Whether `tombstone`'s replacer is still known (pending, `live`, or buried): it then stays
    /// with that replacer's tree.
    fn replacer_known(&self, txid: &Txid, live: &impl Fn(&Txid) -> bool) -> bool {
        // A chain that loops (or runs past the walk's bound) ends nowhere pending: none known.
        let mut remaining = MAX_WALK;
        self.newest_known_version(*txid, &mut remaining, live)
            .is_some_and(|root| root != *txid)
    }

    /// The invariants the type states that its counts and indexes keep: `bytes` sums the bodies,
    /// `replaced` counts the `Replaced` tombstones, and `predecessors_by_replacer[by]` lists
    /// exactly those replaced by `by`. O(tombstones), for debug builds.
    pub(super) fn consistent(&self) -> bool {
        let (mut bytes, mut replaced) = (0, 0);
        for (txid, tombstone) in self.tombstones.iter() {
            bytes += tombstone.record.memory();
            if let Some(by) = tombstone.replaced_by() {
                replaced += 1;
                if !self
                    .predecessors_by_replacer
                    .get(by)
                    .is_some_and(|predecessors| predecessors.contains(txid))
                {
                    return false;
                }
            }
        }
        let listed = self
            .predecessors_by_replacer
            .iter()
            .try_fold(0, |listed, (by, predecessors)| {
                predecessors
                    .iter()
                    .all(|txid| {
                        self.tombstones.get(txid).and_then(TxTombstone::replaced_by) == Some(by)
                    })
                    .then_some(listed + predecessors.len())
            });
        bytes == self.bytes && replaced == self.replaced && listed == Some(replaced)
    }

    /// Lets go of what's past its retention, then of the oldest past [`MAX_MEMORY`]. Retention: a
    /// lone tombstone after [`RETENTION`]; a replaced version with its tree; a tree's newest
    /// version, once it isn't `live`, [`TREE_RETENTION`] after it left, taking the tree. Past the
    /// cap: the oldest lone tombstones first, then the oldest replaced versions, each with what it
    /// replaced, then the oldest trees. O(k) in what leaves the queues; stale entries
    /// (re-burials, exhumes) are skipped.
    pub fn evict_old(&mut self, live: impl Fn(&Txid) -> bool) {
        self.evict_order(&live, false);
        self.evict_trees(&live, false);
        self.evict_order(&live, true);
        // A replaced version stays listed as long as its tombstone: its entry leaves when stale,
        // or with it past the cap. The listing reads this queue: a change to it is a change
        // readers see.
        let listed = self.replacements.len();
        while let Some(&(t, txid)) = self.replacements.front() {
            let stale = self.current(&txid, t).is_none();
            if !stale && self.bytes <= MAX_MEMORY {
                break;
            }
            self.replacements.pop_front();
            if !stale {
                self.remove_tree(txid);
            }
        }
        self.evict_trees(&live, true);
        // Stale entries behind a current head (a long-pending tree's, an hour's burials): dropped
        // once they outnumber the current.
        let tombstones = &self.tombstones;
        let current = |(t, txid): &(Instant, Txid)| {
            tombstones
                .get(txid)
                .is_some_and(|tombstone| tombstone.removed_at == *t)
        };
        if self.replacements.len() > 2 * self.replaced + 1_024 {
            self.replacements.retain(current);
        }
        if self.order.len() > 2 * self.tombstones.len() + 1_024 {
            self.order.retain(current);
        }
        if self.replacements.len() != listed {
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// `order`'s front past [`RETENTION`] (`over_cap`: or while past [`MAX_MEMORY`]): a version
    /// whose replacer is still known stays with its tree, a tree's newest version moves to
    /// `trees`, any other leaves.
    fn evict_order(&mut self, live: &impl Fn(&Txid) -> bool, over_cap: bool) {
        while let Some(&(t, txid)) = self.order.front() {
            // A stale head leaves whatever its age.
            let current = self.current(&txid, t).is_some();
            if current && t.elapsed() < RETENTION && !(over_cap && self.bytes > MAX_MEMORY) {
                break;
            }
            self.order.pop_front();
            if !current || self.replacer_known(&txid, live) {
                continue;
            }
            // A tree's head, or a version replaced by one the pool never saw (a conflict a block
            // confirmed: the tree is rooted there), stays a tree's time.
            let replaced = self
                .tombstones
                .get(&txid)
                .is_some_and(|tomb| tomb.replaced_by().is_some());
            if replaced || self.predecessors_by_replacer.contains_key(&txid) {
                self.trees.push_back((t, txid));
            } else {
                self.remove(&txid);
            }
        }
    }

    /// `trees`' front past [`TREE_RETENTION`] (`over_cap`: or while past [`MAX_MEMORY`]): each
    /// takes its tree, unless pending again. A head Core lists again but the pool hasn't loaded
    /// yet (after a restart, or past the fetch cap) keeps it, `held`: its arrival exhumes it,
    /// its predecessors staying with it.
    fn evict_trees(&mut self, live: &impl Fn(&Txid) -> bool, over_cap: bool) {
        let mut held = mem::take(&mut self.held);
        held.retain(|&(t, txid)| {
            if self.current(&txid, t).is_none() || self.replacer_known(&txid, live) {
                return false;
            }
            if live(&txid) {
                return true;
            }
            self.remove_tree(txid);
            false
        });
        self.held = held;
        while let Some(&(t, txid)) = self.trees.front() {
            if t.elapsed() < TREE_RETENTION && !(over_cap && self.bytes > MAX_MEMORY) {
                break;
            }
            self.trees.pop_front();
            if self.current(&txid, t).is_none() || self.replacer_known(&txid, live) {
                continue;
            }
            if live(&txid) {
                self.held.push((t, txid));
            } else {
                self.remove_tree(txid);
            }
        }
    }
}
