use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use brk_types::{FeeRate, Txid};
use rapidhash::RapidHashMap;
use smallvec::SmallVec;

pub mod tombstone;

pub use tombstone::TxTombstone;

use crate::{TxRemoval, stores::tx_store::TxRecord};

/// How long a tombstone outside any replacement tree stays.
const RETENTION: Duration = Duration::from_hours(1);
/// How long a replacement tree stays once its newest version left the pool (mempool.space's):
/// while that one is pending, the whole tree does.
const TREE_RETENTION: Duration = Duration::from_hours(24);
/// The bodies the graveyard holds at most (serialized bytes): past it, the oldest lone tombstones
/// go first, then the oldest trees, then the oldest replaced versions of pending ones (a flood of
/// cheap replacements can't grow it further).
const MAX_BYTES: u64 = 256 << 20;
/// The longest replacement chain an eviction check follows (the RBF listings' work bound).
const MAX_WALK: usize = 4_096;

/// Recently-dropped txs retained for reappearance (a returning tx keeps its
/// first sighting) and post-mine analytics (RBF/replacement chains, etc.).
#[derive(Clone, Default)]
pub struct TxGraveyard {
    revision: u64,
    tombstones: RapidHashMap<Txid, TxTombstone>,
    predecessors_by_replacer: RapidHashMap<Txid, SmallVec<[Txid; 1]>>,
    /// Every burial in order: a lone tombstone leaves from here, a tree's newest version moves on
    /// to `trees`, a replaced version waits for its tree.
    order: VecDeque<(Instant, Txid)>,
    /// The newest versions of trees no longer pending, in burial order: each takes its tree.
    trees: VecDeque<(Instant, Txid)>,
    /// The `Replaced` burials of `order`, so RBF listings never scan the far
    /// more numerous mined and dropped txs.
    replacements: VecDeque<(Instant, Txid)>,
    /// The serialized size of every body held.
    bytes: u64,
    /// How many `Replaced` tombstones are held.
    replaced: usize,
}

impl TxGraveyard {
    pub fn revision(&self) -> u64 {
        self.revision
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
                .entry(replacer)
                .or_default()
                .push(txid);
            self.replacements.push_back((removed_at, txid));
        }
        self.order.push_back((removed_at, txid));
        self.revision = self.revision.wrapping_add(1);
    }

    fn remove_predecessor(&mut self, txid: &Txid, tombstone: &TxTombstone) {
        let Some(replacer) = tombstone.replaced_by() else {
            return;
        };
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

    /// Transactions a block confirmed: a replaced version buried among them was mined after all.
    /// Its tombstone says so from now on (a lone one, or its tree's newest), which also ends a
    /// cycle its replacer's confirmed conflict closed.
    pub fn mark_mined<'a>(&mut self, txids: impl IntoIterator<Item = &'a Txid>) {
        let now = Instant::now();
        for txid in txids {
            let Some(tombstone) = self.tombstones.get(txid) else {
                continue;
            };
            if tombstone.replaced_by().is_none() {
                continue;
            }
            let previous = tombstone.clone();
            self.remove_predecessor(txid, &previous);
            let tombstone = self.tombstones.get_mut(txid).unwrap();
            tombstone.removal = TxRemoval::Mined;
            tombstone.removed_at = now;
            self.replaced -= 1;
            self.order.push_back((now, *txid));
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Lets go of what's past its retention, and of the oldest past [`MAX_BYTES`]: a lone
    /// tombstone after [`RETENTION`]; a replaced version with its tree; a tree's newest version,
    /// once it isn't `live`, [`TREE_RETENTION`] after it left, taking the tree. O(k) in what
    /// leaves the queues; stale entries (re-burials, exhumes) are skipped.
    pub fn evict_old(&mut self, live: impl Fn(&Txid) -> bool) {
        while let Some(&(t, txid)) = self.order.front() {
            if t.elapsed() < RETENTION && self.bytes <= MAX_BYTES {
                break;
            }
            self.order.pop_front();
            if self.current(&txid, t).is_none() || self.replacer_known(&txid, &live) {
                continue;
            }
            if self.predecessors_by_replacer.contains_key(&txid) {
                self.trees.push_back((t, txid));
            } else {
                self.remove(&txid);
            }
        }
        while let Some(&(t, txid)) = self.trees.front() {
            if t.elapsed() < TREE_RETENTION && self.bytes <= MAX_BYTES {
                break;
            }
            self.trees.pop_front();
            if self.current(&txid, t).is_some() && !self.replacer_known(&txid, &live) {
                self.remove_tree(txid);
            }
        }
        // A replaced version stays listed as long as its tombstone: its entry leaves when stale,
        // or with it past the cap (the oldest first, so a tree loses its earliest versions). The
        // listing reads this queue: a change to it is a change readers see.
        let listed = self.replacements.len();
        while let Some(&(t, txid)) = self.replacements.front() {
            let stale = self.current(&txid, t).is_none();
            if !stale && self.bytes <= MAX_BYTES {
                break;
            }
            self.replacements.pop_front();
            if !stale {
                self.remove(&txid);
            }
        }
        // Stale entries behind a long-pending tree's: dropped once they outnumber the current.
        if self.replacements.len() > 2 * self.replaced + 1_024 {
            let tombstones = &self.tombstones;
            self.replacements.retain(|(t, txid)| {
                tombstones
                    .get(txid)
                    .is_some_and(|tombstone| tombstone.removed_at == *t)
            });
        }
        if self.replacements.len() != listed {
            self.revision = self.revision.wrapping_add(1);
        }
    }
}
