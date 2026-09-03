use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use brk_types::{FeeRate, Transaction, Txid};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

mod tombstone;

pub use tombstone::TxTombstone;

use crate::{TxRemoval, state::TxEntry};

const RETENTION: Duration = Duration::from_hours(1);

/// Recently-dropped txs retained for reappearance detection (Puller can revive
/// them without RPC) and post-mine analytics (RBF/replacement chains, etc.).
#[derive(Default)]
pub struct TxGraveyard {
    tombstones: FxHashMap<Txid, TxTombstone>,
    predecessors_by_replacer: FxHashMap<Txid, SmallVec<[Txid; 1]>>,
    order: VecDeque<(Instant, Txid)>,
}

impl TxGraveyard {
    pub fn tombstones_len(&self) -> usize {
        self.tombstones.len()
    }

    pub fn order_len(&self) -> usize {
        self.order.len()
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
    pub fn replacement_root_of(&self, mut txid: Txid) -> Txid {
        while let Some(TxRemoval::Replaced { by }) = self.tombstones.get(&txid).map(|t| &t.removal)
        {
            txid = *by;
        }
        txid
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
    /// `removed_at == t` check skips those.
    pub fn replaced_iter_recent_first(&self) -> impl Iterator<Item = (&Txid, &Txid)> {
        self.order.iter().rev().filter_map(|(t, txid)| {
            let ts = self.tombstones.get(txid)?;
            if ts.removed_at != *t {
                return None;
            }
            Some((txid, ts.replaced_by()?))
        })
    }

    pub fn bury(
        &mut self,
        tx: Transaction,
        entry: TxEntry,
        chunk_rate: FeeRate,
        removal: TxRemoval,
    ) {
        let txid = entry.txid;
        let removed_at = Instant::now();
        let tombstone = TxTombstone {
            tx,
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

    /// Test-only: force the oldest `order` entries to look older than
    /// `RETENTION`. Splits `Instant::now()` arithmetic out of the test
    /// bodies and avoids real-time sleeps.
    #[cfg(test)]
    fn shift_oldest_back(&mut self, count: usize) {
        let bumped = Instant::now() - (RETENTION + Duration::from_secs(1));
        for entry in self.order.iter_mut().take(count) {
            let txid = entry.1;
            entry.0 = bumped;
            if let Some(ts) = self.tombstones.get_mut(&txid) {
                ts.removed_at = bumped;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{FeeRate, MempoolEntryInfo, Sats, Timestamp, VSize, Weight};

    use super::*;
    use crate::test_support::{fake_tx, fake_txid};

    fn tomb_inputs(seed: u8) -> (Transaction, TxEntry, FeeRate) {
        let tx = fake_tx(seed, &[], &[]);
        let info = MempoolEntryInfo {
            txid: tx.txid,
            vsize: VSize::from(100u64),
            weight: Weight::from(400u64),
            fee: Sats::from(100u64),
            first_seen: Timestamp::from(0u32),
            depends: vec![],
        };
        let entry = TxEntry::new(&info, 100, false);
        let rate = FeeRate::from((Sats::from(100u64), VSize::from(100u64)));
        (tx, entry, rate)
    }

    #[test]
    fn bury_then_exhume_roundtrips_the_tombstone() {
        let mut g = TxGraveyard::default();
        let (tx, entry, rate) = tomb_inputs(1);
        let txid = entry.txid;
        g.bury(tx, entry, rate, TxRemoval::Vanished);
        assert_eq!(g.tombstones_len(), 1);
        assert!(g.get(&txid).is_some());

        let resurrected = g.exhume(&txid).expect("tombstone present");
        assert_eq!(resurrected.entry.txid, txid);
        assert!(g.get(&txid).is_none());
        assert_eq!(g.tombstones_len(), 0);
        // `order` still references the exhumed entry until evict_old
        // runs. The timestamp-match check on evict skips stale rows.
        assert_eq!(g.order_len(), 1);
    }

    #[test]
    fn get_vanished_filters_out_replaced_tombstones() {
        let mut g = TxGraveyard::default();
        let (tx_a, entry_a, rate) = tomb_inputs(2);
        let (tx_b, entry_b, _) = tomb_inputs(3);
        let txid_a = entry_a.txid;
        let txid_b = entry_b.txid;
        g.bury(tx_a, entry_a, rate, TxRemoval::Replaced { by: txid_b });
        g.bury(tx_b, entry_b, rate, TxRemoval::Vanished);

        assert!(g.get_vanished(&txid_a).is_none());
        assert!(g.get_vanished(&txid_b).is_some());
    }

    #[test]
    fn replacement_root_walks_replaced_chain() {
        let mut g = TxGraveyard::default();
        let (tx_a, entry_a, rate) = tomb_inputs(4);
        let (tx_b, entry_b, _) = tomb_inputs(5);
        let (tx_c, entry_c, _) = tomb_inputs(6);
        let a = entry_a.txid;
        let b = entry_b.txid;
        let c = entry_c.txid;
        g.bury(tx_a, entry_a, rate, TxRemoval::Replaced { by: b });
        g.bury(tx_b, entry_b, rate, TxRemoval::Replaced { by: c });
        g.bury(tx_c, entry_c, rate, TxRemoval::Vanished);

        assert_eq!(g.replacement_root_of(a), c);
        assert_eq!(g.replacement_root_of(c), c);

        let unknown = fake_txid(99);
        assert_eq!(g.replacement_root_of(unknown), unknown);
    }

    #[test]
    fn predecessors_of_returns_direct_replacers() {
        let mut g = TxGraveyard::default();
        let (tx_a, entry_a, rate) = tomb_inputs(7);
        let (tx_b, entry_b, _) = tomb_inputs(8);
        let (tx_c, entry_c, _) = tomb_inputs(9);
        let replacer = entry_c.txid;
        let a = entry_a.txid;
        let b = entry_b.txid;
        g.bury(tx_a, entry_a, rate, TxRemoval::Replaced { by: replacer });
        g.bury(tx_b, entry_b, rate, TxRemoval::Replaced { by: replacer });
        g.bury(tx_c, entry_c, rate, TxRemoval::Vanished);

        let mut preds: Vec<Txid> = g.predecessors_of(&replacer).map(|(t, _)| *t).collect();
        preds.sort_unstable_by_key(|t| t.as_slice()[0]);
        let mut expected = vec![a, b];
        expected.sort_unstable_by_key(|t| t.as_slice()[0]);
        assert_eq!(preds, expected);

        assert_eq!(g.predecessors_of(&fake_txid(123)).count(), 0);

        g.exhume(&a).unwrap();
        assert_eq!(g.predecessors_of(&replacer).count(), 1);
        assert_eq!(
            g.predecessors_of(&replacer).next().map(|(t, _)| *t),
            Some(b)
        );
    }

    #[test]
    fn re_bury_moves_predecessor_to_new_replacer() {
        let mut g = TxGraveyard::default();
        let (tx, entry, rate) = tomb_inputs(20);
        let predecessor = entry.txid;
        let first = fake_txid(21);
        let second = fake_txid(22);

        g.bury(
            tx.clone(),
            entry.clone(),
            rate,
            TxRemoval::Replaced { by: first },
        );
        g.bury(tx, entry, rate, TxRemoval::Replaced { by: second });

        assert_eq!(g.predecessors_of(&first).count(), 0);
        assert_eq!(
            g.predecessors_of(&second).next().map(|(t, _)| *t),
            Some(predecessor)
        );
    }

    #[test]
    fn eviction_removes_predecessor_index_entry() {
        let mut g = TxGraveyard::default();
        let (tx, entry, rate) = tomb_inputs(23);
        let replacer = fake_txid(24);
        g.bury(tx, entry, rate, TxRemoval::Replaced { by: replacer });
        g.shift_oldest_back(1);
        g.evict_old();

        assert_eq!(g.predecessors_of(&replacer).count(), 0);
    }

    #[test]
    fn replaced_iter_recent_first_skips_stale_order_entries() {
        let mut g = TxGraveyard::default();
        let (tx_a, entry_a, rate) = tomb_inputs(10);
        let (tx_b, entry_b, _) = tomb_inputs(11);
        let replacer = entry_b.txid;
        let pred = entry_a.txid;
        g.bury(
            tx_a.clone(),
            entry_a.clone(),
            rate,
            TxRemoval::Replaced { by: replacer },
        );
        g.bury(tx_b, entry_b, rate, TxRemoval::Vanished);

        // Re-bury the predecessor: its `order` entry is now stale.
        g.bury(tx_a, entry_a, rate, TxRemoval::Replaced { by: replacer });

        let collected: Vec<(Txid, Txid)> = g
            .replaced_iter_recent_first()
            .map(|(p, by)| (*p, *by))
            .collect();
        assert_eq!(collected, vec![(pred, replacer)]);
    }

    #[test]
    fn evict_old_drops_aged_tombstones() {
        let mut g = TxGraveyard::default();
        let (tx_a, entry_a, rate) = tomb_inputs(12);
        let (tx_b, entry_b, _) = tomb_inputs(13);
        let txid_a = entry_a.txid;
        let txid_b = entry_b.txid;
        g.bury(tx_a, entry_a, rate, TxRemoval::Vanished);
        g.bury(tx_b, entry_b, rate, TxRemoval::Vanished);

        g.shift_oldest_back(1);
        g.evict_old();

        assert!(g.get(&txid_a).is_none(), "aged tombstone evicted");
        assert!(g.get(&txid_b).is_some(), "fresh tombstone retained");
    }

    #[test]
    fn re_bury_mid_retention_resets_age() {
        let mut g = TxGraveyard::default();
        let (tx, entry, rate) = tomb_inputs(14);
        let txid = entry.txid;
        g.bury(tx.clone(), entry.clone(), rate, TxRemoval::Vanished);
        g.shift_oldest_back(1);

        // Re-bury: a stale order entry remains pointing at the old time,
        // but `removed_at` on the tombstone is now fresh. evict_old's
        // timestamp-match check should drop the stale order entry without
        // touching the live tombstone.
        g.bury(tx, entry, rate, TxRemoval::Vanished);
        g.evict_old();
        assert!(g.get(&txid).is_some());
    }
}
