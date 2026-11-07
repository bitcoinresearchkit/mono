//! Letting go of tombstones: past their retention, then past the memory cap.

use std::time::Duration;

use brk_types::Txid;
use rapidhash::RapidHashSet;

use super::{Moment, TxGraveyard, TxTombstone};
use crate::stores::cow::CowQueue;

/// How long a lone tombstone stays.
const RETENTION: Duration = Duration::from_hours(1);
/// How long a replacement tree stays once its root left the pool (mempool.space's): while that
/// one is pending, the whole tree does.
const TREE_RETENTION: Duration = Duration::from_hours(24);
/// The memory the graveyard's bodies take at most ([`TxRecord::memory`]). Past it, the oldest lone
/// tombstones go first: they cost nothing to make (an eviction burst, or a node restarting with an
/// old mempool.dat, buries hundreds of thousands at once) and are kept an hour only. Then the oldest
/// replaced versions, each with what it replaced, then the oldest trees. Only a flood of
/// replacements pushes out older trees' histories, at a price: every replacement pays Core's
/// incremental relay fee over its size, about 0.2 BTC to fill it.
///
/// [`TxRecord::memory`]: crate::stores::tx_store::TxRecord::memory
const MAX_MEMORY: u64 = 256 << 20;
/// Stale entries a queue may hold past twice the current ones before it drops them all.
const STALE_SLACK: usize = 1_024;

impl TxGraveyard {
    /// Whether `(at, txid)` is a lone tombstone's current entry: one filed then, replacing none
    /// and replaced by none.
    fn lone_at(&self, txid: &Txid, at: Moment) -> bool {
        self.filed.get(txid) == Some(&at)
            && !self.predecessors_by_replacer.contains_key(txid)
            && self
                .tombstones
                .get(txid)
                .is_some_and(|tombstone| tombstone.replaced_by().is_none())
    }

    /// Whether `(at, txid)` is a root's current entry: one filed then, replacing others and
    /// replaced by none.
    fn root_at(&self, txid: &Txid, at: Moment) -> bool {
        self.filed.get(txid) == Some(&at)
            && self.predecessors_by_replacer.contains_key(txid)
            && self
                .tombstones
                .get(txid)
                .is_none_or(|tombstone| tombstone.replaced_by().is_none())
    }

    fn over_cap(&self) -> bool {
        self.bytes > MAX_MEMORY
    }

    /// The invariants the type states: `bytes` sums the bodies, `replaced` counts the `Replaced`
    /// tombstones, `predecessors_by_replacer[by]` lists exactly those replaced by `by`, the links
    /// close no cycle, and `filed` holds exactly the lone tombstones and the roots, each with its
    /// queue's entry. O(tombstones), for debug builds and tests.
    pub(crate) fn consistent(&self) -> bool {
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
        let lone = filed(&self.lone, |txid, at| self.lone_at(txid, at));
        let roots = filed(&self.trees, |txid, at| self.root_at(txid, at));
        let every_lone_filed = self.tombstones.iter().all(|(txid, tombstone)| {
            tombstone.replaced_by().is_some()
                || self.predecessors_by_replacer.contains_key(txid)
                || lone.contains(txid)
        });
        let every_root_filed = self.predecessors_by_replacer.iter().all(|(by, _)| {
            self.tombstones
                .get(by)
                .is_some_and(|tombstone| tombstone.replaced_by().is_some())
                || roots.contains(by)
        });
        bytes == self.bytes
            && replaced == self.replaced
            && listed == Some(replaced)
            && every_lone_filed
            && every_root_filed
            && self.filed.len() == lone.len() + roots.len()
            && self.acyclic()
    }

    /// Lets go of what's past its retention as of `now`, then of the oldest past [`MAX_MEMORY`].
    /// Retention: a lone tombstone [`RETENTION`] after it became lone; a tree, once its root
    /// isn't `live` (Core lists it, loaded or not), [`TREE_RETENTION`] after it left (a root seen
    /// pending is looked at again a retention later). Past the cap: the oldest lone tombstones,
    /// then the oldest replaced versions, each with what it replaced, then the oldest trees no
    /// longer pending. O(what leaves the queues).
    pub fn evict_old(&mut self, now: Moment, live: impl Fn(&Txid) -> bool) {
        let listed = self.replacements.len();
        self.evict_lone(now, false);
        self.evict_trees(now, &live, false);
        self.evict_replaced(now, false);
        // Pruning a tree's versions can leave its root lone: the passes go again until under
        // the cap, or until one frees nothing (what's left is pending).
        while self.over_cap() {
            let bytes = self.bytes;
            self.evict_lone(now, true);
            self.evict_replaced(now, true);
            self.evict_trees(now, &live, true);
            if self.bytes == bytes {
                break;
            }
        }
        self.drop_stale();
        // The listing reads `replacements`: a change to it is a change readers see.
        if self.replacements.len() != listed {
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// `replacements`' stale front (a replaced version stays listed as long as its tombstone;
    /// `over_cap`: and the oldest versions while past [`MAX_MEMORY`], each with what it replaced).
    fn evict_replaced(&mut self, now: Moment, over_cap: bool) {
        while let Some(&(at, txid)) = self.replacements.front() {
            let stale = self.current(&txid, at).is_none();
            if !stale && !(over_cap && self.over_cap()) {
                break;
            }
            self.replacements.pop_front();
            if !stale {
                self.remove_tree(txid, now);
            }
        }
    }

    /// `lone`'s front past [`RETENTION`] (`over_cap`: or while past [`MAX_MEMORY`]), and every
    /// stale entry before the first that stays.
    fn evict_lone(&mut self, now: Moment, over_cap: bool) {
        while let Some(&(at, txid)) = self.lone.front() {
            let lone = self.lone_at(&txid, at);
            let due = now.saturating_duration_since(at) >= RETENTION || over_cap && self.over_cap();
            if lone && !due {
                break;
            }
            self.lone.pop_front();
            if lone {
                self.remove(&txid, now);
            }
        }
    }

    /// `trees`' front past [`TREE_RETENTION`] (`over_cap`: or while past [`MAX_MEMORY`]): each
    /// goes whole, unless its root is pending again (`live`), when it's looked at again a
    /// retention later. Every stale entry before the first that stays goes.
    fn evict_trees(&mut self, now: Moment, live: &impl Fn(&Txid) -> bool, over_cap: bool) {
        // A pass over the cap looks at each entry once: the pending ones it puts back.
        let mut looked = self.trees.len();
        while let Some(&(at, root)) = self.trees.front() {
            let current = self.root_at(&root, at);
            let due = now.saturating_duration_since(at) >= TREE_RETENTION
                || over_cap && self.over_cap() && looked > 0;
            if current && !due {
                break;
            }
            self.trees.pop_front();
            looked = looked.saturating_sub(1);
            if !current {
                continue;
            }
            if live(&root) {
                self.file_root(root, now);
            } else {
                self.remove_tree(root, now);
            }
        }
    }

    /// Drops every queue's stale entries once they outnumber the current ones.
    fn drop_stale(&mut self) {
        if self.lone.len() > 2 * self.tombstones.len() + STALE_SLACK {
            let mut lone = std::mem::take(&mut self.lone);
            lone.retain(|(at, txid)| self.lone_at(txid, *at));
            self.lone = lone;
        }
        if self.trees.len() > 2 * self.predecessors_by_replacer.len() + STALE_SLACK {
            let mut trees = std::mem::take(&mut self.trees);
            trees.retain(|(at, txid)| self.root_at(txid, *at));
            self.trees = trees;
        }
        if self.replacements.len() > 2 * self.replaced + STALE_SLACK {
            let mut replacements = std::mem::take(&mut self.replacements);
            replacements.retain(|(at, txid)| self.current(txid, *at).is_some());
            self.replacements = replacements;
        }
    }
}

/// The txids of `queue`'s entries `valid` holds current.
fn filed(
    queue: &CowQueue<(Moment, Txid)>,
    valid: impl Fn(&Txid, Moment) -> bool,
) -> RapidHashSet<Txid> {
    queue
        .iter()
        .filter(|(at, txid)| valid(txid, *at))
        .map(|(_, txid)| *txid)
        .collect()
}
