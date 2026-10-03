use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use brk_error::{Error, Result};
use brk_types::{FeeRate, Transaction, Txid};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

pub mod tombstone;

pub use tombstone::TxTombstone;

use crate::{TxRemoval, state::TxEntry};

const RETENTION: Duration = Duration::from_hours(1);

/// Recently-dropped txs retained for reappearance detection (Puller can revive
/// them without RPC) and post-mine analytics (RBF/replacement chains, etc.).
#[derive(Clone, Default)]
pub struct TxGraveyard {
    revision: u64,
    tombstones: FxHashMap<Txid, TxTombstone>,
    predecessors_by_replacer: FxHashMap<Txid, SmallVec<[Txid; 1]>>,
    order: VecDeque<(Instant, Txid)>,
}

impl TxGraveyard {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn get(&self, txid: &Txid) -> Option<&TxTombstone> {
        self.tombstones.get(txid)
    }

    /// Tombstone iff the tx vanished from the pool (mined, expired, or
    /// dropped). `Replaced` tombstones return `None` because the tx
    /// will not confirm.
    pub fn get_vanished(&self, txid: &Txid) -> Option<&TxTombstone> {
        let tomb = self.tombstones.get(txid)?;
        matches!(tomb.removal, TxRemoval::Vanished).then_some(tomb)
    }

    /// Walk forward through `Replaced { by }` to the terminal replacer.
    /// Returns the first txid in the chain that isn't a `Replaced`
    /// tombstone: live, `Vanished`, or unknown (chain broken because an
    /// intermediate `by` aged out of the graveyard).
    /// Every step consumes the caller's shared budget, so cycles also terminate.
    pub fn replacement_root_of(&self, mut txid: Txid, remaining: &mut usize) -> Result<Txid> {
        loop {
            *remaining = remaining
                .checked_sub(1)
                .ok_or(Error::Internal("RBF history exceeds traversal limit"))?;
            match self.tombstones.get(&txid).map(|t| &t.removal) {
                Some(TxRemoval::Replaced { by }) => txid = *by,
                _ => return Ok(txid),
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
    /// `order` may carry stale entries (re-buries, prior exhumes). The
    /// `removed_at == t` check marks those as `None`, so callers can charge
    /// every scanned entry against their work budget, including stale entries.
    pub fn replacement_candidates_recent_first(
        &self,
    ) -> impl Iterator<Item = Option<(&Txid, &Txid)>> {
        self.order.iter().rev().map(|(t, txid)| {
            let ts = self.tombstones.get(txid)?;
            if ts.removed_at != *t {
                return None;
            }
            Some((txid, ts.replaced_by()?))
        })
    }

    pub fn bury(
        &mut self,
        tx: impl Into<Arc<Transaction>>,
        entry: TxEntry,
        chunk_rate: FeeRate,
        removal: TxRemoval,
    ) {
        let txid = entry.txid;
        let removed_at = Instant::now();
        let tombstone = TxTombstone {
            tx: tx.into(),
            entry,
            chunk_rate,
            removal,
            removed_at,
        };
        let replacer = tombstone.replaced_by().copied();
        if let Some(previous) = self.tombstones.insert(txid, tombstone) {
            self.remove_predecessor(&txid, &previous);
        }
        if let Some(replacer) = replacer {
            self.predecessors_by_replacer
                .entry(replacer)
                .or_default()
                .push(txid);
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

    /// Remove and return the tombstone, e.g. when the tx comes back to life.
    pub fn exhume(&mut self, txid: &Txid) -> Option<TxTombstone> {
        let tombstone = self.tombstones.remove(txid)?;
        self.revision = self.revision.wrapping_add(1);
        self.remove_predecessor(txid, &tombstone);
        Some(tombstone)
    }

    /// Drop tombstones older than RETENTION. O(k) in the number of evictions.
    ///
    /// The order queue may carry stale entries (from re-buries or prior
    /// exhumes). The timestamp-match check skips those without disturbing
    /// live tombstones.
    pub fn evict_old(&mut self) {
        while let Some(&(t, _)) = self.order.front() {
            if t.elapsed() < RETENTION {
                break;
            }
            let (_, txid) = self.order.pop_front().unwrap();
            self.revision = self.revision.wrapping_add(1);
            let should_remove = self
                .tombstones
                .get(&txid)
                .is_some_and(|tombstone| tombstone.removed_at == t);
            if should_remove {
                let tombstone = self.tombstones.remove(&txid).unwrap();
                self.remove_predecessor(&txid, &tombstone);
            }
        }
    }
}
