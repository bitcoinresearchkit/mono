//! Transactions that recently left the pool: kept for RBF trees, removal reasons and the first
//! sighting of a transaction that returns.
//!
//! # Model
//!
//! A tombstone records why its transaction left (its removal). Its removal and the links between
//! tombstones decide its kind, and its kind decides how long it stays:
//!
//! | Removal               | Replaced others | Kind    | Stays                                      |
//! |-----------------------|-----------------|---------|--------------------------------------------|
//! | `Replaced { by }`     | either          | version | as long as its tree                        |
//! | `Mined` or `Vanished` | no              | lone    | one hour after becoming lone               |
//! | `Mined` or `Vanished` | yes             | root    | its tree, one day after the root left      |
//!
//! A tree is a root and the versions linked to it. A root may have no tombstone: it can be a
//! pending replacer, or a confirmed conflict the pool never held (an "unseen" root). A tree stays
//! while Core still lists its root. Past the memory cap, lone tombstones go first, then the oldest
//! versions, then the oldest trees ([`TxGraveyard::evict_old`]).
//!
//! # Transitions
//!
//! | Event                                                 | Method    | Result                       |
//! |-------------------------------------------------------|-----------|------------------------------|
//! | A transaction leaves the pool                         | `bury`    | a new tombstone              |
//! | A transaction returns to the pool                     | `exhume`  | no tombstone; what it replaced stays linked |
//! | A block mines a buried transaction                    | `confirm` | `Mined`                      |
//! | A reorg removes the block that mined it, and the new blocks don't mine it | `confirm` | `Replaced` by a conflict the new blocks confirmed, else `Vanished` |
//! | A reorg removes an unseen replacer for good           | `confirm` | what it replaced: as the row above |
//! | A block confirms a conflict of something that vanished | `confirm` | `Replaced` by that conflict  |
//! | A replacer the fetch cap held back arrives            | `relink`  | what it conflicts with: `Replaced` by it |
//! | Retention ends, or the memory cap is passed           | `evict_old` | gone                       |
//!
//! A `Vanished` tombstone becomes `Replaced` only when a confirmed conflict or a late replacer
//! shows it was displaced. See [`Linker`] for which burials stay eligible and for how long.

use brk_types::{BlockHash, FeeRate, Transaction, Txid};
use rapidhash::{RapidHashSet, fast::RandomState};
use smallvec::SmallVec;

mod eviction;
pub mod file;
mod linker;
mod moment;
pub mod tombstone;
mod upkeep;

pub use file::GraveyardFile;
use linker::Linker;
pub use moment::Moment;
pub use tombstone::TxTombstone;
pub use upkeep::Upkeep;
use upkeep::{Kind, Retention};

use crate::{
    TxRemoval,
    steps::Confirmed,
    stores::{
        cow::{CowMap, CowQueue},
        tx_store::TxRecord,
    },
};

/// The tombstones, shared with publications. The module docs describe the model.
///
/// What only the writer needs lives in [`Upkeep`], which every change takes: when each tombstone is
/// due, and which vanished ones may still turn out replaced.
///
/// Invariants, checked by [`Self::consistent`] in debug builds and by a randomized test:
///
/// - `predecessors_by_replacer[by]` lists exactly the `Replaced { by }` tombstones, in the order
///   they became so, oldest first.
/// - `replaced` counts those tombstones, and `bytes` sums the memory of every body.
/// - `Replaced` links form no cycle: a replacer with a tombstone is exhumed or marked mined in the
///   cycle that links it.
/// - `replacements` has one entry per `Replaced` tombstone, in burial order. An entry whose tombstone
///   was buried again since is stale, and readers skip it.
#[derive(Clone, Default)]
pub struct TxGraveyard {
    revision: u64,
    tombstones: CowMap<Txid, TxTombstone, RandomState>,
    predecessors_by_replacer: CowMap<Txid, SmallVec<[Txid; 1]>, RandomState>,
    /// The `Replaced` burials, so RBF listings never scan the far more numerous mined and dropped
    /// txs.
    replacements: CowQueue<(Moment, Txid)>,
    /// The memory all bodies take ([`TxRecord::memory`]).
    bytes: u64,
    /// How many `Replaced` tombstones are held.
    replaced: usize,
    /// The chain tip up to which blocks were applied (`confirm`). The next cycle reads the blocks
    /// after it, including after a restart: it is saved with the graveyard.
    confirmed_tip: Option<BlockHash>,
    /// When `confirmed_tip` was last set. Blocks after that tip can't have displaced anything
    /// buried before this moment.
    confirmed_at: Option<Moment>,
}

impl TxGraveyard {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn confirmed_tip(&self) -> Option<BlockHash> {
        self.confirmed_tip
    }

    /// When [`Self::confirmed_tip`] was last set.
    pub fn confirmed_at(&self) -> Option<Moment> {
        self.confirmed_at
    }

    /// Sets the confirmed tip and its time as read back from disk.
    fn restore_confirmed_tip(&mut self, tip: BlockHash, at: Moment) {
        self.confirmed_tip = Some(tip);
        self.confirmed_at = Some(at);
        self.revision = self.revision.wrapping_add(1);
    }

    /// The tombstones that vanished after `confirmed_at`, each with its burial time. Blocks after
    /// the confirmed tip may still show them displaced ([`Upkeep::of`]).
    pub fn vanished_since_confirmed(&self) -> Vec<(Txid, Moment)> {
        self.tombstones
            .iter()
            .filter(|(_, tombstone)| {
                matches!(tombstone.removal, TxRemoval::Vanished)
                    && self.confirmed_at.is_none_or(|at| tombstone.removed_at > at)
            })
            .map(|(txid, tombstone)| (*txid, tombstone.removed_at))
            .collect()
    }

    /// Records that every block up to `tip` was applied.
    pub fn set_confirmed_tip(&mut self, tip: BlockHash, upkeep: &mut Upkeep) {
        self.confirmed_at = Some(Moment::now());
        upkeep.linker.blocks_confirmed();
        if self.confirmed_tip != Some(tip) {
            self.confirmed_tip = Some(tip);
            self.revision = self.revision.wrapping_add(1);
        }
    }

    pub fn get(&self, txid: &Txid) -> Option<&TxTombstone> {
        self.tombstones.get(txid)
    }

    /// `txid`'s tombstone, unless it is `Replaced`.
    pub fn get_unreplaced(&self, txid: &Txid) -> Option<&TxTombstone> {
        let tomb = self.tombstones.get(txid)?;
        tomb.replaced_by().is_none().then_some(tomb)
    }

    /// `txid`'s replacer, if that is still known: pending (`live`) or buried. One step of the walk
    /// to a tree's newest version (`api::rbf`'s `tree_root`).
    pub fn next_version(&self, txid: &Txid, live: impl Fn(&Txid) -> bool) -> Option<Txid> {
        self.tombstones
            .get(txid)
            .and_then(TxTombstone::replaced_by)
            .filter(|by| live(by) || self.tombstones.contains_key(by))
            .copied()
    }

    /// The tombstones `replacer` directly replaced, oldest first.
    pub fn predecessors_of<'a>(
        &'a self,
        replacer: &'a Txid,
    ) -> impl DoubleEndedIterator<Item = (&'a Txid, &'a TxTombstone)> {
        self.predecessors_by_replacer
            .get(replacer)
            .into_iter()
            .flatten()
            .filter_map(|txid| self.tombstones.get(txid).map(|tombstone| (txid, tombstone)))
    }

    /// Every `Replaced` tombstone as (replaced txid, replacer txid), most recently buried first.
    /// Stale entries yield `None`, so callers can charge them against a work budget too.
    pub fn replacements_recent_first(&self) -> impl Iterator<Item = Option<(&Txid, &Txid)>> {
        self.replacements.iter().rev().map(|(t, txid)| {
            let ts = self.tombstones.get(txid)?;
            if ts.removed_at != *t {
                return None;
            }
            Some((txid, ts.replaced_by()?))
        })
    }

    /// Buries `record`. A vanished one is also noted in the [`Linker`], since a later block or
    /// arrival may show what displaced it.
    pub fn bury(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        upkeep: &mut Upkeep,
    ) {
        let (txid, now) = (record.entry.txid, Moment::now());
        upkeep.linker.undefer(&txid);
        self.insert(record, chunk_rate, removal, now, &mut upkeep.retention);
        if matches!(removal, TxRemoval::Vanished) {
            upkeep.linker.vanished(txid, now);
        }
    }

    /// Inserts a tombstone read from disk, with its original burial time. Call in burial order,
    /// oldest first, because `replacements` keeps time order. [`Upkeep::of`] schedules retention
    /// afterwards.
    fn restore(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        removed_at: Moment,
    ) {
        self.insert(record, chunk_rate, removal, removed_at, &mut Retention::default());
    }

    fn insert(
        &mut self,
        record: TxRecord,
        chunk_rate: FeeRate,
        removal: TxRemoval,
        removed_at: Moment,
        retention: &mut Retention,
    ) {
        let txid = record.entry.txid;
        self.bytes += record.memory();
        let tombstone = TxTombstone {
            record,
            chunk_rate,
            removal,
            removed_at,
        };
        if let Some(previous) = self.tombstones.insert(txid, tombstone) {
            self.bytes -= previous.record.memory();
            if let Some(by) = previous.replaced_by() {
                self.replaced -= 1;
                self.unlist(&txid, by, removed_at, retention);
            }
        }
        self.index_removal(txid, removal, removed_at, retention);
        self.revision = self.revision.wrapping_add(1);
    }

    /// Records that `txid`'s tombstone got `removal` at `at`. A `Replaced` one joins its replacer's
    /// predecessors. Any other is scheduled as a root if it replaced others, or as lone.
    fn index_removal(
        &mut self,
        txid: Txid,
        removal: TxRemoval,
        at: Moment,
        retention: &mut Retention,
    ) {
        if let TxRemoval::Replaced { by } = removal {
            self.replaced += 1;
            retention.unschedule(&txid);
            self.list(txid, by, at, retention);
            self.replacements.push_back((at, txid));
        } else if self.predecessors_by_replacer.contains_key(&txid) {
            retention.schedule(txid, Kind::Root, at);
        } else {
            retention.schedule(txid, Kind::Lone, at);
        }
    }

    /// Adds `txid` to `by`'s predecessors. If those were empty, `by` becomes a root scheduled from
    /// `at`, unless `by` is itself a version (its tree's root is the one scheduled).
    fn list(&mut self, txid: Txid, by: Txid, at: Moment, retention: &mut Retention) {
        let predecessors = self
            .predecessors_by_replacer
            .shard_mut(&by)
            .entry(by)
            .or_default();
        let first = predecessors.is_empty();
        predecessors.push(txid);
        if first && self.get(&by).is_none_or(|tombstone| tombstone.replaced_by().is_none()) {
            retention.schedule(by, Kind::Root, at);
        }
    }

    /// Removes `txid` from `by`'s predecessors. If none remain, `by` is no longer a root: with an
    /// unreplaced tombstone it becomes lone, scheduled from `at`, and without a tombstone it is
    /// unscheduled. A `by` that is itself a version stays as it is.
    fn unlist(&mut self, txid: &Txid, by: &Txid, at: Moment, retention: &mut Retention) {
        let emptied = self
            .predecessors_by_replacer
            .get_mut(by)
            .is_some_and(|predecessors| {
                predecessors.retain(|predecessor| predecessor != txid);
                predecessors.is_empty()
            });
        if !emptied {
            return;
        }
        self.predecessors_by_replacer.remove(by);
        match self.get(by).map(|tombstone| tombstone.replaced_by().is_none()) {
            Some(true) => retention.schedule(*by, Kind::Lone, at),
            Some(false) => {}
            None => retention.unschedule(by),
        }
    }

    /// Removes the tombstone of a transaction that returned to the pool. What it replaced stays
    /// linked, because its replacer is pending again.
    pub fn exhume(&mut self, txid: &Txid, upkeep: &mut Upkeep) {
        self.remove(txid, Moment::now(), &mut upkeep.retention);
    }

    /// Removes `txid`'s tombstone, if any. If it replaced others, `txid` stays a root, scheduled
    /// from `at`, now without a tombstone.
    fn remove(
        &mut self,
        txid: &Txid,
        at: Moment,
        retention: &mut Retention,
    ) -> Option<TxTombstone> {
        let tombstone = self.tombstones.remove(txid)?;
        self.revision = self.revision.wrapping_add(1);
        self.bytes -= tombstone.record.memory();
        if let Some(by) = tombstone.replaced_by() {
            self.replaced -= 1;
            self.unlist(txid, by, at, retention);
        }
        if self.predecessors_by_replacer.contains_key(txid) {
            retention.schedule(*txid, Kind::Root, at);
        } else {
            retention.unschedule(txid);
        }
        Some(tombstone)
    }

    /// Removes `txid`'s tombstone and everything it replaced, recursively: a whole tree, or the
    /// part of one below a version.
    fn remove_tree(&mut self, txid: Txid, at: Moment, retention: &mut Retention) {
        let mut stack = vec![txid];
        while let Some(txid) = stack.pop() {
            if let Some(predecessors) = self.predecessors_by_replacer.get(&txid) {
                stack.extend(predecessors.iter().copied());
            }
            self.remove(&txid, at, retention);
        }
    }

    /// The tombstone buried at `at`, unless a later burial or an exhume replaced it.
    fn current(&self, txid: &Txid, at: Moment) -> Option<&TxTombstone> {
        self.tombstones
            .get(txid)
            .filter(|tombstone| tombstone.removed_at == at)
    }

    /// Applies new blocks to the tombstones.
    ///
    /// - A buried transaction the blocks mined becomes `Mined`, even a replaced version. That also breaks
    ///   a cycle its replacer's confirmed conflict closed.
    /// - A `Mined` transaction whose block a reorg removed and the new blocks didn't mine becomes
    ///   `Replaced` by a conflict the new blocks confirmed, or else `Vanished`.
    /// - What an unseen replacer replaced gets the same treatment when a reorg removes that
    ///   replacer and it is neither mined again nor back in Core's pool (`live`).
    /// - A `Vanished` transaction a confirmed conflict displaced becomes `Replaced` by that
    ///   conflict. Two groups are eligible: burials since the blocks were last all applied
    ///   ([`Linker::since_confirmed`]), and burials whose late replacer was listed but never loaded
    ///   ([`Linker::displaced`], which a restart forgets). An outpoint spent again after its
    ///   transaction was evicted or expired replaced nothing.
    ///
    /// O(the blocks' transactions, plus what vanished since then or awaits a late replacer).
    pub fn confirm(&mut self, blocks: &Confirmed, live: impl Fn(&Txid) -> bool, upkeep: &mut Upkeep) {
        let now = Moment::now();
        let mut changes: Vec<(Txid, TxRemoval)> = Vec::new();
        for txid in blocks.mined_txids() {
            if self
                .get(txid)
                .is_some_and(|tombstone| !matches!(tombstone.removal, TxRemoval::Mined))
            {
                changes.push((*txid, TxRemoval::Mined));
            }
        }
        let conflict = |tombstone: &TxTombstone| {
            let tx = &tombstone.record.tx;
            blocks
                .conflicts(&tx.txid, tx.input.iter().map(|txin| (&txin.txid, txin.vout)))
                .next()
        };
        for txid in blocks.disconnected_txids() {
            if let Some(tombstone) = self.get(txid)
                && matches!(tombstone.removal, TxRemoval::Mined)
                && !blocks.mined(txid)
            {
                let removal = conflict(tombstone)
                    .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by });
                changes.push((*txid, removal));
            }
            if self.get(txid).is_none() && !blocks.mined(txid) && !live(txid) {
                for predecessor in self.predecessors_by_replacer.get(txid).into_iter().flatten() {
                    if let Some(tombstone) = self.get(predecessor)
                        && !blocks.mined(predecessor)
                    {
                        let removal = conflict(tombstone)
                            .map_or(TxRemoval::Vanished, |by| TxRemoval::Replaced { by });
                        changes.push((*predecessor, removal));
                    }
                }
            }
        }
        let mut displaced: Vec<(Txid, Moment, Txid)> = upkeep
            .linker
            .since_confirmed()
            .iter()
            .filter_map(|&(txid, at)| {
                let tombstone = self.current(&txid, at)?;
                Some((txid, at, conflict(tombstone)?))
            })
            .collect();
        displaced.extend(upkeep.linker.displaced(|outpoint| blocks.spender(outpoint)));
        let mut reclassified: RapidHashSet<Txid> = changes.iter().map(|(txid, _)| *txid).collect();
        for (txid, at, by) in displaced {
            if !blocks.mined(&txid) && Self::is_vanished(&self.tombstones, &txid, at) && reclassified.insert(txid) {
                changes.push((txid, TxRemoval::Replaced { by }));
            }
        }
        for (txid, removal) in changes {
            self.reclassify(txid, removal, now, upkeep);
        }
    }

    /// Changes a tombstone's removal as of `now`. Its retention starts over.
    fn reclassify(&mut self, txid: Txid, removal: TxRemoval, now: Moment, upkeep: &mut Upkeep) {
        let Some(tombstone) = self.tombstones.get_mut(&txid) else {
            return;
        };
        let previous = tombstone.replaced_by().copied();
        tombstone.removal = removal;
        tombstone.removed_at = now;
        if let Some(by) = previous {
            self.replaced -= 1;
            self.unlist(&txid, &by, now, &mut upkeep.retention);
        }
        self.index_removal(txid, removal, now, &mut upkeep.retention);
        if matches!(removal, TxRemoval::Vanished) {
            upkeep.linker.vanished(txid, now);
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// A replacer the fetch cap held back has arrived. What it conflicts with, among what vanished
    /// no earlier than the cycle it was first listed in, becomes `Replaced` by it
    /// ([`Linker::losers`]). Call after `confirm`, and only when every block was read: a confirmed
    /// conflict takes precedence.
    pub fn relink(&mut self, arrival: &Transaction, upkeep: &mut Upkeep) {
        let now = Moment::now();
        let tombstones = &self.tombstones;
        let losers = upkeep
            .linker
            .losers(arrival, |txid, at| Self::is_vanished(tombstones, txid, at));
        for loser in losers {
            self.reclassify(loser, TxRemoval::Replaced { by: arrival.txid }, now, upkeep);
        }
    }

    /// Relinks this cycle's arrivals (`entered`) and those held back earlier. With a block unread
    /// (`all_read` false), holds this cycle's back instead, since a confirmed conflict takes
    /// precedence. `arrival` gives a pooled transaction's body.
    pub fn relink_arrivals<'a>(
        &mut self,
        entered: &[Txid],
        all_read: bool,
        arrival: impl Fn(&Txid) -> Option<&'a Transaction>,
        upkeep: &mut Upkeep,
    ) {
        if !all_read {
            entered.iter().for_each(|txid| upkeep.linker.defer(*txid));
            return;
        }
        for txid in upkeep.linker.deferred().iter().chain(entered) {
            if let Some(tx) = arrival(txid) {
                self.relink(tx, upkeep);
            }
        }
        upkeep.linker.clear_deferred();
    }

    /// Indexes this cycle's vanished tombstones by the outpoints they spent. Call before
    /// [`Self::relink`], so an arrival in the same cycle finds them.
    pub fn index_vanished(&self, upkeep: &mut Upkeep) {
        let tombstones = &self.tombstones;
        upkeep.linker.index(|txid, at| {
            tombstones
                .get(txid)
                .filter(|tombstone| tombstone.removed_at == at)
                .map(|tombstone| tombstone.record.tx.input.as_slice())
        });
    }

    /// Ends the cycle. `unloaded` holds the listed transactions it couldn't load
    /// ([`Linker::end_cycle`]).
    pub fn end_cycle(&self, unloaded: &RapidHashSet<Txid>, upkeep: &mut Upkeep) {
        let tombstones = &self.tombstones;
        upkeep
            .linker
            .end_cycle(unloaded, |txid, at| Self::is_vanished(tombstones, txid, at));
        debug_assert!(self.consistent(upkeep), "graveyard invariants");
    }

    /// Whether no chain of `Replaced { by }` links returns to where it started. O(tombstones).
    fn acyclic(&self) -> bool {
        let mut done: RapidHashSet<Txid> = RapidHashSet::default();
        for (start, _) in self.tombstones.iter() {
            let mut path: RapidHashSet<Txid> = RapidHashSet::default();
            let mut at = *start;
            while !done.contains(&at) {
                if !path.insert(at) {
                    return false;
                }
                match self.get(&at).and_then(TxTombstone::replaced_by) {
                    Some(by) => at = *by,
                    None => break,
                }
            }
            done.extend(path);
        }
        true
    }

    /// Whether `txid`'s tombstone is the vanished one buried at `at`.
    fn is_vanished(
        tombstones: &CowMap<Txid, TxTombstone, RandomState>,
        txid: &Txid,
        at: Moment,
    ) -> bool {
        tombstones.get(txid).is_some_and(|tombstone| {
            tombstone.removed_at == at && matches!(tombstone.removal, TxRemoval::Vanished)
        })
    }
}
