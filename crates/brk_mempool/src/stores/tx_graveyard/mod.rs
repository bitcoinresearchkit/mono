use std::{
    mem,
    time::{Duration, Instant},
};

use brk_types::{BlockHash, FeeRate, Transaction, Txid, Vout};
use rapidhash::{RapidHashMap, RapidHashSet, fast::RandomState};
use smallvec::SmallVec;

pub mod file;
pub mod tombstone;

pub use file::GraveyardFile;
pub use tombstone::TxTombstone;

use crate::{
    TxRemoval,
    stores::{
        cow::{CowMap, CowQueue},
        tx_store::TxRecord,
    },
};

/// How long a tombstone outside any replacement tree stays.
const RETENTION: Duration = Duration::from_hours(1);
/// How long a replacement tree stays once its newest version left the pool (mempool.space's):
/// while that one is pending, the whole tree does.
const TREE_RETENTION: Duration = Duration::from_hours(24);
/// The bodies the graveyard holds at most (serialized bytes): past it, the oldest replaced
/// versions go first, each with what it replaced (a flood of cheap replacements can't grow it
/// further, nor push out the rest), then the oldest trees, then the oldest lone tombstones.
const MAX_BYTES: u64 = 256 << 20;
/// The longest replacement chain an eviction check follows (the RBF listings' work bound).
const MAX_WALK: usize = 4_096;

/// A vanished burial waiting for a late replacer: its tombstone's time, and its cycle.
#[derive(Clone, Copy)]
struct Burial {
    txid: Txid,
    at: Instant,
    cycle: u64,
}

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
/// entries at their fronts, and all of them once they outnumber the current ones.
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
    /// The serialized size of every body held.
    bytes: u64,
    /// How many `Replaced` tombstones are held.
    replaced: usize,
    /// What vanished in cycles whose view wasn't complete, by the outpoints each spent (each with
    /// its burial's time and cycle): a replacer the per-cycle fetch cap held back still links it
    /// when it arrives. Cleared by the first complete cycle.
    unlinked: RapidHashMap<(Txid, Vout), SmallVec<[Burial; 1]>>,
    /// This cycle's vanished burials (and reclassifications), for `unlinked` unless the cycle's
    /// view is complete.
    vanished: Vec<(Txid, Instant)>,
    /// Listed transactions not loaded yet, by the cycle each was first listed in: only one listed
    /// by the cycle something vanished in can be what replaced it.
    unloaded_since: RapidHashMap<Txid, u64>,
    /// The current cycle's number.
    cycle: u64,
    /// The tip whose blocks it was last told (`confirm`): the next cycle reads the blocks since,
    /// and so does one after a restart (it is saved with it).
    told: Option<BlockHash>,
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

    /// Every block up to `tip` told: a change publications and saves carry.
    pub fn set_told(&mut self, tip: BlockHash) {
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

    /// Walk forward through `Replaced { by }` to the terminal replacer: the first txid in the
    /// chain that isn't a `Replaced` tombstone (live, or buried otherwise), or the last one whose
    /// replacer is neither `live` nor buried (aged out, or a conflict a block confirmed without
    /// the pool seeing it): the history the pool still knows stays rooted there.
    /// Every step consumes the caller's shared budget, so cycles also terminate;
    /// `None` once it runs out.
    pub fn replacement_root_of(
        &self,
        mut txid: Txid,
        remaining: &mut usize,
        live: impl Fn(&Txid) -> bool,
    ) -> Option<Txid> {
        loop {
            *remaining = remaining.checked_sub(1)?;
            match self.tombstones.get(&txid).map(|t| &t.removal) {
                Some(TxRemoval::Replaced { by })
                    if live(by) || self.tombstones.contains_key(by) =>
                {
                    txid = *by
                }
                _ => return Some(txid),
            }
        }
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
            self.vanished.push((txid, removed_at));
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
        self.bytes += record.entry.size;
        let tombstone = TxTombstone {
            record,
            chunk_rate,
            removal,
            removed_at,
        };
        let replacer = tombstone.replaced_by().copied();
        self.replaced += usize::from(replacer.is_some());
        if let Some(previous) = self.tombstones.insert(txid, tombstone) {
            self.bytes -= previous.record.entry.size;
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
        self.bytes -= tombstone.record.entry.size;
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

    /// Whether `tombstone`'s replacer is still known (pending, `live`, or buried): it then stays
    /// with that replacer's tree.
    fn replacer_known(&self, txid: &Txid, live: &impl Fn(&Txid) -> bool) -> bool {
        // A chain that loops (or runs past the walk's bound) ends nowhere pending: none known.
        let mut remaining = MAX_WALK;
        self.replacement_root_of(*txid, &mut remaining, live)
            .is_some_and(|root| root != *txid)
    }

    /// What new blocks confirmed, told to the buried: one they confirmed reads mined from now on (a
    /// replaced version mined after all, which also ends a cycle its replacer's confirmed conflict
    /// closed, or one buried vanished), and a vanished one a confirmed conflict displaced reads
    /// replaced by it. A cycle that couldn't read its blocks buried them vanished; the next cycle's
    /// read tells them here. One mined by a block a reorg took off the chain (`disconnected`) that
    /// neither returned nor was mined again reads vanished, or replaced by a conflict the new
    /// blocks confirmed. O(tombstones and the vanished ones' inputs), on cycles with new blocks
    /// only.
    pub fn confirm(
        &mut self,
        mined: impl Fn(&Txid) -> bool,
        disconnected: impl Fn(&Txid) -> bool,
        conflict: impl Fn(&TxTombstone) -> Option<Txid>,
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
                    removal => match conflict(tombstone) {
                        Some(by) => TxRemoval::Replaced { by },
                        None if matches!(removal, TxRemoval::Mined) => TxRemoval::Vanished,
                        None => return None,
                    },
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
            TxRemoval::Vanished => self.vanished.push((txid, now)),
            TxRemoval::Mined => {}
        }
        self.order.push_back((now, txid));
        self.revision = self.revision.wrapping_add(1);
    }

    /// Whether `txid`'s tombstone is the vanished one buried at `at`.
    fn vanished_at(&self, txid: &Txid, at: Instant) -> bool {
        self.current(txid, at)
            .is_some_and(|tombstone| matches!(tombstone.removal, TxRemoval::Vanished))
    }

    /// An arrival the fetch cap held back, replacing what vanished meanwhile: what it conflicts
    /// with that vanished no earlier than it was first listed reads replaced by it (one listed
    /// only later, spending an outpoint the vanished one freed, replaced nothing). Called after
    /// the cycle's blocks are told (`confirm`), and only when they all could be: a confirmed
    /// conflict comes first.
    pub fn relink(&mut self, arrival: &Transaction) {
        if self.unlinked.is_empty() {
            return;
        }
        // One never left unloaded was first listed this cycle.
        let listed = self
            .unloaded_since
            .get(&arrival.txid)
            .copied()
            .unwrap_or(self.cycle);
        let now = Instant::now();
        let mut relinked: SmallVec<[Txid; 1]> = SmallVec::new();
        for txin in &arrival.input {
            let Some(losers) = self.unlinked.get(&(txin.txid, txin.vout)) else {
                continue;
            };
            for burial in losers {
                // Several of its inputs can lead to one loser: told once.
                if listed <= burial.cycle
                    && burial.txid != arrival.txid
                    && !relinked.contains(&burial.txid)
                    && self.vanished_at(&burial.txid, burial.at)
                {
                    relinked.push(burial.txid);
                }
            }
        }
        for loser in relinked {
            self.reclassify(loser, TxRemoval::Replaced { by: arrival.txid }, now);
        }
    }

    /// This cycle's vanished (buried, or told so), indexed by the outpoints they spent: before
    /// [`Self::relink`], so a same-cycle arrival finds them too.
    pub fn index_vanished(&mut self) {
        for (txid, at) in mem::take(&mut self.vanished) {
            let Some(tombstone) = self
                .tombstones
                .get(&txid)
                .filter(|tombstone| tombstone.removed_at == at)
            else {
                continue;
            };
            for txin in &tombstone.record.tx.input {
                self.unlinked
                    .entry((txin.txid, txin.vout))
                    .or_default()
                    .push(Burial {
                        txid,
                        at,
                        cycle: self.cycle,
                    });
            }
        }
    }

    /// The cycle's end, with the listed transactions it couldn't load. None: no replacer of what
    /// vanished is still to arrive, all is let go. Otherwise what's no longer vanished lets go,
    /// and the unloaded keep when they were first listed.
    pub fn end_cycle(&mut self, unloaded: &RapidHashSet<Txid>) {
        if unloaded.is_empty() {
            self.unlinked.clear();
            self.vanished.clear();
            self.unloaded_since.clear();
        } else {
            let mut unlinked = mem::take(&mut self.unlinked);
            unlinked.retain(|_, losers| {
                losers.retain(|burial| self.vanished_at(&burial.txid, burial.at));
                !losers.is_empty()
            });
            self.unlinked = unlinked;
            let previous = mem::take(&mut self.unloaded_since);
            self.unloaded_since = unloaded
                .iter()
                .map(|txid| (*txid, previous.get(txid).copied().unwrap_or(self.cycle)))
                .collect();
        }
        self.cycle += 1;
    }

    /// Lets go of what's past its retention, then of the oldest past [`MAX_BYTES`]. Retention: a
    /// lone tombstone after [`RETENTION`]; a replaced version with its tree; a tree's newest
    /// version, once it isn't `live`, [`TREE_RETENTION`] after it left, taking the tree. Past the
    /// cap: the oldest replaced versions first, each with what it replaced (a flood of cheap
    /// replacements pays for itself before anything else), then the oldest trees, then the oldest
    /// lone tombstones. O(k) in what leaves the queues; stale entries (re-burials, exhumes) are
    /// skipped.
    pub fn evict_old(&mut self, live: impl Fn(&Txid) -> bool) {
        self.evict_order(&live, false);
        self.evict_trees(&live, false);
        // A replaced version stays listed as long as its tombstone: its entry leaves when stale,
        // or with it past the cap. The listing reads this queue: a change to it is a change
        // readers see.
        let listed = self.replacements.len();
        while let Some(&(t, txid)) = self.replacements.front() {
            let stale = self.current(&txid, t).is_none();
            if !stale && self.bytes <= MAX_BYTES {
                break;
            }
            self.replacements.pop_front();
            if !stale {
                self.remove_tree(txid);
            }
        }
        self.evict_trees(&live, true);
        self.evict_order(&live, true);
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

    /// `order`'s front past [`RETENTION`] (`over_cap`: or while past [`MAX_BYTES`]): a version
    /// whose replacer is still known stays with its tree, a tree's newest version moves to
    /// `trees`, any other leaves.
    fn evict_order(&mut self, live: &impl Fn(&Txid) -> bool, over_cap: bool) {
        while let Some(&(t, txid)) = self.order.front() {
            // A stale head leaves whatever its age.
            let current = self.current(&txid, t).is_some();
            if current && t.elapsed() < RETENTION && !(over_cap && self.bytes > MAX_BYTES) {
                break;
            }
            self.order.pop_front();
            if !current || self.replacer_known(&txid, live) {
                continue;
            }
            if self.predecessors_by_replacer.contains_key(&txid) {
                self.trees.push_back((t, txid));
            } else {
                self.remove(&txid);
            }
        }
    }

    /// `trees`' front past [`TREE_RETENTION`] (`over_cap`: or while past [`MAX_BYTES`]): each
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
            if t.elapsed() < TREE_RETENTION && !(over_cap && self.bytes > MAX_BYTES) {
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
