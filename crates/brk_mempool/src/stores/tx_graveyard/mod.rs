use std::time::Instant;

use brk_types::{BlockHash, FeeRate, Transaction, Txid};
use rapidhash::{RapidHashSet, fast::RandomState};
use smallvec::SmallVec;

mod eviction;
pub mod file;
mod linker;
pub mod tombstone;

pub use file::GraveyardFile;
use linker::Linker;
pub use tombstone::TxTombstone;

use crate::{
    TxRemoval,
    stores::{
        cow::{CowMap, CowQueue},
        tx_store::TxRecord,
    },
};

/// Recently-dropped txs retained for reappearance (a returning tx keeps its
/// first sighting) and post-mine analytics (RBF/replacement chains, etc.).
///
/// Invariants: each burial or reclassification appends `(removed_at, txid)` to `order` (and, a
/// `Replaced` one, to `replacements`); an entry whose time isn't its tombstone's `removed_at` is
/// stale (a later burial or reclassification, or an exhume) and skipped wherever read. Past
/// retention, `order`'s front leaves: a version whose replacer is still known stays with that
/// replacer's tree, a tree's newest version moves to `trees` (taking its tree when it goes), any
/// other goes. `predecessors_by_replacer[by]` lists exactly the `Replaced { by }` tombstones,
/// `replaced` counts them, `bytes` sums the held bodies. `order` and `replacements` drop stale
/// entries at their fronts, and all of them once they outnumber the current ones. What vanished
/// while the view wasn't complete waits for a late replacer in the [`Linker`], which holds no
/// tombstone of its own.
#[derive(Clone, Default)]
pub struct TxGraveyard {
    revision: u64,
    tombstones: CowMap<Txid, TxTombstone, RandomState>,
    predecessors_by_replacer: CowMap<Txid, SmallVec<[Txid; 1]>, RandomState>,
    /// Every burial in order: a lone tombstone leaves from here, a tree's newest version moves on
    /// to `trees`, a replaced version waits for its tree.
    order: CowQueue<(Instant, Txid)>,
    /// The newest versions of trees no longer pending, in burial order: each takes its tree.
    trees: CowQueue<(Instant, Txid)>,
    /// The `Replaced` burials of `order`, so RBF listings never scan the far
    /// more numerous mined and dropped txs.
    replacements: CowQueue<(Instant, Txid)>,
    /// The memory every body held takes ([`TxRecord::memory`]).
    bytes: u64,
    /// How many `Replaced` tombstones are held.
    replaced: usize,
    /// What vanished while the view wasn't complete, for a late replacer to link.
    linker: Linker,
    /// The tip whose blocks it was last told (`confirm`): the next cycle reads the blocks since,
    /// and so does one after a restart (it is saved with it).
    told: Option<BlockHash>,
    /// When every block up to `told` was last told: what was buried before, the blocks since
    /// didn't displace.
    told_at: Option<Instant>,
    /// Tree heads past retention that Core lists again but the pool hasn't loaded: checked every
    /// pass (`trees` stays in time order), each tree going once its head is neither.
    held: Vec<(Instant, Txid)>,
}

impl TxGraveyard {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn told(&self) -> Option<BlockHash> {
        self.told
    }

    /// When every block up to [`Self::told`] was last told.
    pub fn told_at(&self) -> Option<Instant> {
        self.told_at
    }

    /// `told` and `told_at` read back from disk.
    fn restore_told(&mut self, tip: BlockHash, at: Option<Instant>) {
        self.told = Some(tip);
        self.told_at = at;
        self.revision = self.revision.wrapping_add(1);
    }

    /// Every block up to `tip` told: a change publications and saves carry.
    pub fn set_told(&mut self, tip: BlockHash) {
        self.told_at = Some(Instant::now());
        if self.told != Some(tip) {
            self.told = Some(tip);
            self.revision = self.revision.wrapping_add(1);
        }
    }

    pub fn get(&self, txid: &Txid) -> Option<&TxTombstone> {
        self.tombstones.get(txid)
    }

    /// Tombstone iff the tx left the pool without a replacer (mined, expired, or dropped);
    /// `Replaced` tombstones are excluded.
    pub fn get_unreplaced(&self, txid: &Txid) -> Option<&TxTombstone> {
        let tomb = self.tombstones.get(txid)?;
        tomb.replaced_by().is_none().then_some(tomb)
    }

    /// The newest version of `txid`'s history the pool still knows, following each replacer
    /// while it is pending (`live`) or buried: the first that isn't a `Replaced` tombstone, or
    /// the last whose replacer is neither (aged out, or a conflict a block confirmed without the
    /// pool seeing it). An RBF tree's root is this version, unless its tombstone names a
    /// replacer the pool never saw, which then roots it (`api::rbf`'s `tree_root`). Every step
    /// consumes the caller's budget, so a loop ends too: `None` once it runs out.
    pub fn newest_known_version(
        &self,
        mut txid: Txid,
        remaining: &mut usize,
        live: impl Fn(&Txid) -> bool,
    ) -> Option<Txid> {
        loop {
            *remaining = remaining.checked_sub(1)?;
            match self.next_version(&txid, &live) {
                Some(by) => txid = by,
                None => return Some(txid),
            }
        }
    }

    /// `txid`'s replacer, when it is still known (`live`, or buried): one step of a walk to the
    /// root.
    pub fn next_version(&self, txid: &Txid, live: impl Fn(&Txid) -> bool) -> Option<Txid> {
        self.tombstones
            .get(txid)
            .and_then(TxTombstone::replaced_by)
            .filter(|by| live(by) || self.tombstones.contains_key(by))
            .copied()
    }

    /// Tombstones marked as `Replaced { by: replacer }`. Used to walk
    /// backward through RBF history: given a tx that's still live (or
    /// in the graveyard), find every tx it displaced.
    pub fn predecessors_of<'a>(
        &'a self,
        replacer: &'a Txid,
    ) -> impl Iterator<Item = (&'a Txid, &'a TxTombstone)> {
        self.predecessors_by_replacer
            .get(replacer)
            .into_iter()
            .flatten()
            .filter_map(|txid| self.tombstones.get(txid).map(|tombstone| (txid, tombstone)))
    }

    /// Every `Replaced` tombstone, yielded as (`predecessor_txid`,
    /// `replacer_txid`) in reverse bury order (most recent replacement
    /// event first). Caller walks the replacer chain forward to find
    /// each tree's terminal replacer.
    ///
    /// `replacements` may carry stale entries (re-buries, prior exhumes). The
    /// `removed_at == t` check marks those as `None`, so callers can charge
    /// every scanned entry against their work budget, including stale entries.
    pub fn replacements_recent_first(&self) -> impl Iterator<Item = Option<(&Txid, &Txid)>> {
        self.replacements.iter().rev().map(|(t, txid)| {
            let ts = self.tombstones.get(txid)?;
            if ts.removed_at != *t {
                return None;
            }
            Some((txid, ts.replaced_by()?))
        })
    }

    pub fn bury(&mut self, record: TxRecord, chunk_rate: FeeRate, removal: TxRemoval) {
        let txid = record.entry.txid;
        let removed_at = Instant::now();
        self.insert(record, chunk_rate, removal, removed_at);
        if matches!(removal, TxRemoval::Vanished) {
            self.linker.vanished(txid, removed_at);
        }
    }

    /// A tombstone read back from disk, buried when it was (oldest first, as the queues keep
    /// time order).
    fn restore(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        removed_at: Instant,
    ) {
        debug_assert!(
            self.order.iter().next_back().is_none_or(|(t, _)| *t <= removed_at),
            "restored in burial order"
        );
        self.insert(record, chunk_rate, removal, removed_at);
    }

    fn insert(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        removed_at: Instant,
    ) {
        let txid = record.entry.txid;
        self.bytes += record.memory();
        let tombstone = TxTombstone {
            record,
            chunk_rate,
            removal,
            removed_at,
        };
        let replacer = tombstone.replaced_by().copied();
        self.replaced += usize::from(replacer.is_some());
        if let Some(previous) = self.tombstones.insert(txid, tombstone) {
            self.bytes -= previous.record.memory();
            self.replaced -= usize::from(previous.replaced_by().is_some());
            self.remove_predecessor(&txid, &previous);
        }
        if let Some(replacer) = replacer {
            self.predecessors_by_replacer
                .shard_mut(&replacer)
                .entry(replacer)
                .or_default()
                .push(txid);
            self.replacements.push_back((removed_at, txid));
        }
        self.order.push_back((removed_at, txid));
        self.revision = self.revision.wrapping_add(1);
    }

    fn remove_predecessor(&mut self, txid: &Txid, tombstone: &TxTombstone) {
        if let Some(replacer) = tombstone.replaced_by() {
            self.unlist(txid, replacer);
        }
    }

    /// Takes `txid` off `replacer`'s predecessors.
    fn unlist(&mut self, txid: &Txid, replacer: &Txid) {
        let remove_entry =
            self.predecessors_by_replacer
                .get_mut(replacer)
                .is_some_and(|predecessors| {
                    predecessors.retain(|predecessor| predecessor != txid);
                    predecessors.is_empty()
                });
        if remove_entry {
            self.predecessors_by_replacer.remove(replacer);
        }
    }

    /// Drop the tombstone of a tx that came back to life (its predecessors stay: their replacer
    /// is pending again).
    pub fn exhume(&mut self, txid: &Txid) {
        self.remove(txid);
    }

    /// Removes `txid`'s tombstone, if any.
    fn remove(&mut self, txid: &Txid) -> Option<TxTombstone> {
        let tombstone = self.tombstones.remove(txid)?;
        self.revision = self.revision.wrapping_add(1);
        self.bytes -= tombstone.record.memory();
        self.replaced -= usize::from(tombstone.replaced_by().is_some());
        self.remove_predecessor(txid, &tombstone);
        Some(tombstone)
    }

    /// Removes `txid`'s tombstone and every version it replaced, recursively: a tree.
    fn remove_tree(&mut self, txid: Txid) {
        let mut stack = vec![txid];
        while let Some(txid) = stack.pop() {
            if let Some(predecessors) = self.predecessors_by_replacer.get(&txid) {
                stack.extend(predecessors.iter().copied());
            }
            self.remove(&txid);
        }
    }

    /// The tombstone buried at `at`, unless a later burial or an exhume replaced it.
    fn current(&self, txid: &Txid, at: Instant) -> Option<&TxTombstone> {
        self.tombstones
            .get(txid)
            .filter(|tombstone| tombstone.removed_at == at)
    }

    /// What new blocks confirmed, told to the buried: one they confirmed reads mined from now on (a
    /// replaced version mined after all, which also ends a cycle its replacer's confirmed conflict
    /// closed, or one buried vanished), and a vanished one a confirmed conflict displaced reads
    /// replaced by it. Displaced: buried since the blocks were last all told (`told_at`; a cycle
    /// that couldn't read its blocks buried them vanished, the next cycle's read tells them
    /// here), or by a late replacer the pool listed by then and never loaded ([`Linker`], in
    /// memory until the next complete view: a restart or an unread block in between leaves such
    /// a one vanished, as a replacer never seen does). An outpoint respent after its transaction
    /// left (evicted, expired) replaced nothing. One mined
    /// by a block a reorg took off the chain (`disconnected`) that neither returned nor was mined
    /// again reads vanished, or replaced by a conflict the new blocks confirmed. O(tombstones and
    /// the displaceable ones' inputs), on cycles with new blocks only.
    pub fn confirm(
        &mut self,
        mined: impl Fn(&Txid) -> bool,
        disconnected: impl Fn(&Txid) -> bool,
        conflicts: impl Fn(&TxTombstone) -> SmallVec<[Txid; 1]>,
    ) {
        let now = Instant::now();
        let changes: Vec<(Txid, TxRemoval)> = self
            .tombstones
            .iter()
            .filter_map(|(txid, tombstone)| {
                let removal = match tombstone.removal {
                    TxRemoval::Mined if mined(txid) || !disconnected(txid) => return None,
                    _ if mined(txid) => TxRemoval::Mined,
                    TxRemoval::Replaced { .. } => return None,
                    TxRemoval::Mined => conflicts(tombstone)
                        .first()
                        .map_or(TxRemoval::Vanished, |&by| TxRemoval::Replaced { by }),
                    TxRemoval::Vanished => {
                        let recent = self.told_at.is_none_or(|at| tombstone.removed_at > at);
                        if !recent && !self.linker.awaits_late_replacer() {
                            return None;
                        }
                        let inputs = &tombstone.record.tx.input;
                        let by = conflicts(tombstone).into_iter().find(|by| {
                            recent || self.linker.displaced(txid, tombstone.removed_at, inputs, by)
                        })?;
                        TxRemoval::Replaced { by }
                    }
                };
                Some((*txid, removal))
            })
            .collect();
        for (txid, removal) in changes {
            self.reclassify(txid, removal, now);
        }
    }

    /// A tombstone told another reason, as of `now`: its retention starts again.
    fn reclassify(&mut self, txid: Txid, removal: TxRemoval, now: Instant) {
        let Some(tombstone) = self.tombstones.get_mut(&txid) else {
            return;
        };
        let previous = tombstone.replaced_by().copied();
        tombstone.removal = removal;
        tombstone.removed_at = now;
        if let Some(replacer) = previous {
            self.unlist(&txid, &replacer);
            self.replaced -= 1;
        }
        match removal {
            TxRemoval::Replaced { by } => {
                self.predecessors_by_replacer
                    .shard_mut(&by)
                    .entry(by)
                    .or_default()
                    .push(txid);
                self.replacements.push_back((now, txid));
                self.replaced += 1;
            }
            TxRemoval::Vanished => self.linker.vanished(txid, now),
            TxRemoval::Mined => {}
        }
        self.order.push_back((now, txid));
        self.revision = self.revision.wrapping_add(1);
    }

    /// An arrival the fetch cap held back, replacing what vanished meanwhile: what it conflicts
    /// with that vanished no earlier than it was first listed reads replaced by it ([`Linker`]).
    /// Called after the cycle's blocks are told (`confirm`), and only when they all could be: a
    /// confirmed conflict comes first.
    pub fn relink(&mut self, arrival: &Transaction) {
        let now = Instant::now();
        let tombstones = &self.tombstones;
        let losers = self.linker.losers(arrival, |txid, at| Self::is_vanished(tombstones, txid, at));
        for loser in losers {
            self.reclassify(loser, TxRemoval::Replaced { by: arrival.txid }, now);
        }
    }

    /// This cycle's vanished, indexed for [`Self::relink`]: before it, so a same-cycle arrival
    /// finds them too.
    pub fn index_vanished(&mut self) {
        let tombstones = &self.tombstones;
        self.linker.index(|txid, at| {
            tombstones
                .get(txid)
                .filter(|tombstone| tombstone.removed_at == at)
                .map(|tombstone| tombstone.record.tx.input.as_slice())
        });
    }

    /// The cycle's end, with the listed transactions it couldn't load ([`Linker::end_cycle`]).
    pub fn end_cycle(&mut self, unloaded: &RapidHashSet<Txid>) {
        let tombstones = &self.tombstones;
        self.linker
            .end_cycle(unloaded, |txid, at| Self::is_vanished(tombstones, txid, at));
        debug_assert!(self.consistent(), "graveyard invariants");
    }

    /// Whether `txid`'s tombstone is the vanished one buried at `at`.
    fn is_vanished(
        tombstones: &CowMap<Txid, TxTombstone, RandomState>,
        txid: &Txid,
        at: Instant,
    ) -> bool {
        tombstones.get(txid).is_some_and(|tombstone| {
            tombstone.removed_at == at && matches!(tombstone.removal, TxRemoval::Vanished)
        })
    }

}
