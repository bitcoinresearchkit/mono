//! Letting go of tombstones: past their retention, then past the memory cap.

use brk_types::Txid;
use rapidhash::RapidHashSet;

use super::{Kind, Moment, TxGraveyard, TxTombstone, Upkeep};

/// The memory the graveyard's bodies take at most ([`TxRecord::memory`]). Past it, the oldest lone
/// tombstones go first: they cost nothing to make (an eviction burst, or a node restarting with an
/// old mempool.dat, buries hundreds of thousands at once) and are kept an hour only. Then the oldest
/// replaced versions, each with what it replaced, then the oldest trees. Only a flood of
/// replacements pushes out older trees' histories, at a price: every replacement pays Core's
/// incremental relay fee over its size, about 0.2 BTC to fill it.
///
/// [`TxRecord::memory`]: crate::stores::tx_store::TxRecord::memory
const MAX_MEMORY: u64 = 256 << 20;
/// Stale `replacements` entries kept past twice the current ones before they're all dropped.
const STALE_SLACK: usize = 1_024;

impl TxGraveyard {
    fn over_cap(&self) -> bool {
        self.bytes > MAX_MEMORY
    }

    /// The invariants the type states, with `upkeep`'s: `bytes` sums the bodies, `replaced`
    /// counts the `Replaced` tombstones, `predecessors_by_replacer[by]` lists exactly those
    /// replaced by `by`, the links close no cycle, and exactly the lone tombstones and the roots
    /// are filed, each as what it is. O(tombstones), for debug builds and tests.
    pub(crate) fn consistent(&self, upkeep: &Upkeep) -> bool {
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
        let retention = &upkeep.retention;
        let filed: RapidHashSet<&Txid> = self
            .tombstones
            .iter()
            .map(|(txid, _)| txid)
            .chain(self.predecessors_by_replacer.iter().map(|(by, _)| by))
            .filter(|txid| self.kind(txid).is_some())
            .collect();
        let filed_as_what_it_is = filed
            .iter()
            .all(|txid| retention.kind(txid) == self.kind(txid));
        bytes == self.bytes
            && replaced == self.replaced
            && listed == Some(replaced)
            && filed_as_what_it_is
            && retention.len() == filed.len()
            && retention.count(Kind::Lone) + retention.count(Kind::Root) == filed.len()
            && self.acyclic()
    }

    /// Lets go of what's past its retention as of `now`, then of the oldest past [`MAX_MEMORY`].
    /// Retention: a lone tombstone an hour after it became lone; a tree, once its root isn't
    /// `live` (Core lists it, loaded or not), a day after the root left (one seen pending is
    /// looked at again a day later). Past the cap: the oldest lone tombstones, then the oldest
    /// replaced versions, each with what it replaced, then the oldest trees no longer pending.
    /// O(what leaves, times its index's log).
    pub fn evict_old(&mut self, now: Moment, live: impl Fn(&Txid) -> bool, upkeep: &mut Upkeep) {
        let listed = self.replacements.len();
        self.evict_lone(now, false, upkeep);
        self.evict_trees(now, &live, false, upkeep);
        self.evict_replaced(now, false, upkeep);
        // Pruning a tree's versions can leave its root lone: the passes go again until under
        // the cap, or until one frees nothing (what's left is pending).
        while self.over_cap() {
            let bytes = self.bytes;
            self.evict_lone(now, true, upkeep);
            self.evict_replaced(now, true, upkeep);
            self.evict_trees(now, &live, true, upkeep);
            if self.bytes == bytes {
                break;
            }
        }
        if self.replacements.len() > 2 * self.replaced + STALE_SLACK {
            let mut replacements = std::mem::take(&mut self.replacements);
            replacements.retain(|(at, txid)| self.current(txid, *at).is_some());
            self.replacements = replacements;
        }
        // The listing reads `replacements`: a change to it is a change readers see.
        if self.replacements.len() != listed {
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// The lone tombstones due by `now` (`over_cap`: and the oldest while past the cap).
    fn evict_lone(&mut self, now: Moment, over_cap: bool, upkeep: &mut Upkeep) {
        while let Some((due, txid)) = upkeep.retention.first(Kind::Lone) {
            if due > now && !(over_cap && self.over_cap()) {
                break;
            }
            upkeep.retention.unfile(&txid);
            self.remove(&txid, now, &mut upkeep.retention);
        }
    }

    /// The trees due by `now` (`over_cap`: and the oldest while past the cap), each whole, unless
    /// its root is pending again (`live`): that one is looked at again a day later.
    fn evict_trees(
        &mut self,
        now: Moment,
        live: &impl Fn(&Txid) -> bool,
        over_cap: bool,
        upkeep: &mut Upkeep,
    ) {
        // A pass over the cap looks at each once: the pending ones it files again.
        let mut looked = upkeep.retention.count(Kind::Root);
        while let Some((due, root)) = upkeep.retention.first(Kind::Root) {
            if due > now && !(over_cap && self.over_cap() && looked > 0) {
                break;
            }
            looked = looked.saturating_sub(1);
            if live(&root) {
                upkeep.retention.file(root, Kind::Root, now);
                continue;
            }
            upkeep.retention.unfile(&root);
            self.remove_tree(root, now, &mut upkeep.retention);
        }
    }

    /// `replacements`' stale front (a replaced version stays listed as long as its tombstone;
    /// `over_cap`: and the oldest versions while past the cap, each with what it replaced).
    fn evict_replaced(&mut self, now: Moment, over_cap: bool, upkeep: &mut Upkeep) {
        while let Some(&(at, txid)) = self.replacements.front() {
            let stale = self.current(&txid, at).is_none();
            if !stale && !(over_cap && self.over_cap()) {
                break;
            }
            self.replacements.pop_front();
            if !stale {
                self.remove_tree(txid, now, &mut upkeep.retention);
            }
        }
    }
}
