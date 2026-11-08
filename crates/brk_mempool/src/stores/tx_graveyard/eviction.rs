//! Letting go of tombstones: past their retention, then past the memory cap.

use brk_types::Txid;
use rapidhash::RapidHashSet;

use super::{Kind, Moment, TxGraveyard, TxTombstone, Upkeep};

/// The most memory the graveyard's bodies may take
/// ([`TxRecord::memory`](crate::stores::tx_store::TxRecord::memory)). Past it, the oldest lone
/// tombstones go first: they cost nothing to create, and an eviction burst or a node restarting
/// with an old mempool.dat buries hundreds of thousands at once. Then go the oldest versions, each
/// with what it replaced, and then the oldest trees. Only a flood of replacements can push out
/// older trees, and every replacement pays Core's incremental relay fee: about 0.2 BTC to fill
/// this.
const MAX_MEMORY: u64 = 256 << 20;
/// `replacements` is compacted once it holds this many entries more than twice the live ones.
const STALE_SLACK: usize = 1_024;

impl TxGraveyard {
    fn over_cap(&self) -> bool {
        self.bytes > MAX_MEMORY
    }

    /// Whether the invariants hold: those of [`TxGraveyard`], the links form no cycle, and exactly
    /// the lone tombstones and roots are scheduled, each as its kind. O(tombstones): for debug
    /// builds and tests.
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
        let expected: RapidHashSet<&Txid> = self
            .tombstones
            .iter()
            .map(|(txid, _)| txid)
            .chain(self.predecessors_by_replacer.iter().map(|(by, _)| by))
            .filter(|txid| self.kind(txid).is_some())
            .collect();
        let scheduled_as_its_kind = expected
            .iter()
            .all(|txid| retention.kind(txid) == self.kind(txid));
        bytes == self.bytes
            && replaced == self.replaced
            && listed == Some(replaced)
            && scheduled_as_its_kind
            && retention.len() == expected.len()
            && retention.count(Kind::Lone) + retention.count(Kind::Root) == expected.len()
            && self.acyclic()
    }

    /// Drops what is past its retention at `now`, then the oldest past [`MAX_MEMORY`].
    ///
    /// A lone tombstone goes an hour after it became lone. A tree goes a day after its root left,
    /// unless Core still lists the root (`live`, loaded or not); such a tree is checked again a day
    /// later. Past the cap, the order is the oldest lone tombstones, the oldest versions (each with
    /// what it replaced), then the oldest trees no longer pending. O(what leaves, times the log of
    /// its index).
    pub fn evict_old(&mut self, now: Moment, live: impl Fn(&Txid) -> bool, upkeep: &mut Upkeep) {
        let listed = self.replacements.len();
        self.evict_lone(now, false, upkeep);
        self.evict_trees(now, &live, false, upkeep);
        self.evict_replaced(now, false, upkeep);
        // Pruning a tree's versions can leave its root lone, so repeat until under the cap or
        // until a pass frees nothing (all that's left is pending).
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
        // The RBF listing reads `replacements`, so a change to it is visible to readers.
        if self.replacements.len() != listed {
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Drops the lone tombstones due by `now`. With `over_cap`, also the oldest while past the cap.
    fn evict_lone(&mut self, now: Moment, over_cap: bool, upkeep: &mut Upkeep) {
        while let Some((due, txid)) = upkeep.retention.first(Kind::Lone) {
            if due > now && !(over_cap && self.over_cap()) {
                break;
            }
            upkeep.retention.unschedule(&txid);
            self.remove(&txid, now, &mut upkeep.retention);
        }
    }

    /// Drops the trees due by `now`, each whole. With `over_cap`, also the oldest while past the
    /// cap. A tree whose root is pending again (`live`) is rescheduled a day later instead.
    fn evict_trees(
        &mut self,
        now: Moment,
        live: &impl Fn(&Txid) -> bool,
        over_cap: bool,
        upkeep: &mut Upkeep,
    ) {
        // A pass over the cap checks each root once, because pending ones are rescheduled.
        let mut looked = upkeep.retention.count(Kind::Root);
        while let Some((due, root)) = upkeep.retention.first(Kind::Root) {
            if due > now && !(over_cap && self.over_cap() && looked > 0) {
                break;
            }
            looked = looked.saturating_sub(1);
            if live(&root) {
                upkeep.retention.schedule(root, Kind::Root, now);
                continue;
            }
            upkeep.retention.unschedule(&root);
            self.remove_tree(root, now, &mut upkeep.retention);
        }
    }

    /// Drops stale entries at the front of `replacements`. A replaced version stays listed as long
    /// as its tombstone. With `over_cap`, also drops the oldest versions while past the cap, each
    /// with what it replaced.
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
