use std::hash::{Hash, Hasher};

use brk_oracle::HistogramRaw;
use brk_types::{MempoolRecentTx, Transaction, TxOut, Txid, TxidPrefix, Vin};
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashSet, FxHasher};

use crate::{state::TxEntry, stores::LiveHistograms};

mod tx_record;

pub use tx_record::TxRecord;

const RECENT_CAP: usize = 10;

/// Live-pool index keyed by `TxidPrefix`. The full `Txid` lives in
/// `record.entry.txid`, so callers that only have a `Txid` derive the
/// prefix (an 8-byte truncation) at the callsite. `unresolved` is the
/// set of prefixes whose tx still has at least one `prevout: None`,
/// maintained on every `insert` / `remove_by_prefix` / `apply_fills`
/// so the post-update prevout filler can early-exit when empty.
/// `histograms` holds the eligible (oracle-blend) and raw per-bin output
/// histograms, kept in sync on `insert` / `remove_by_prefix` so each read
/// path is a single array clone, not a full pool walk.
#[derive(Default)]
pub struct TxStore {
    records: IndexMap<TxidPrefix, TxRecord, FxBuildHasher>,
    txids_hash: u64,
    content_revision: u64,
    recent: Vec<MempoolRecentTx>,
    unresolved: FxHashSet<TxidPrefix>,
    histograms: LiveHistograms,
}

impl TxStore {
    pub fn contains(&self, txid: &Txid) -> bool {
        self.get(txid).is_some()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.records.reserve(additional);
    }

    pub fn get(&self, txid: &Txid) -> Option<&Transaction> {
        self.record(txid).map(|record| &record.tx)
    }

    pub fn entry(&self, txid: &Txid) -> Option<&TxEntry> {
        self.record(txid).map(|record| &record.entry)
    }

    pub fn record(&self, txid: &Txid) -> Option<&TxRecord> {
        let record = self.records.get(&TxidPrefix::from(txid))?;
        (record.entry.txid == *txid).then_some(record)
    }

    pub fn entry_by_prefix(&self, prefix: &TxidPrefix) -> Option<&TxEntry> {
        self.records.get(prefix).map(|r| &r.entry)
    }

    /// Tx + entry in one map probe. Used by the RBF builder and the
    /// snapshot builder which need both per visited tx.
    pub fn record_by_prefix(&self, prefix: &TxidPrefix) -> Option<&TxRecord> {
        self.records.get(prefix)
    }

    /// `(prefix, record)` pairs in dense storage order. Used by the
    /// snapshot builder to assign a compact `TxIndex` to each live tx.
    pub fn records(&self) -> impl Iterator<Item = (&TxidPrefix, &TxRecord)> {
        self.records.iter()
    }

    pub fn txids(&self) -> impl Iterator<Item = &Txid> {
        self.records.values().map(|r| &r.entry.txid)
    }

    pub fn txids_hash(&self) -> u64 {
        self.txids_hash
    }

    /// Process-local revision for every mutation that can change serialized tx bodies.
    pub fn content_revision(&self) -> u64 {
        self.content_revision
    }

    fn txid_position_hash(txid: &Txid, position: usize) -> u64 {
        let mut hasher = FxHasher::default();
        (position as u64).hash(&mut hasher);
        txid.hash(&mut hasher);
        hasher.finish()
    }

    pub fn insert(&mut self, tx: Transaction, entry: TxEntry) {
        let prefix = entry.txid_prefix();
        debug_assert!(
            !self.records.contains_key(&prefix),
            "TxidPrefix collision: {prefix:?} already mapped. Birthday-rare on SHA-256d."
        );
        let position = self.records.len();
        let txid = entry.txid;
        self.sample_recent(&entry.txid, &tx);
        if tx.input.iter().any(|i| i.prevout.is_none()) {
            self.unresolved.insert(prefix);
        }
        let record = TxRecord { tx, entry };
        self.histograms.add(&record);
        self.records.insert(prefix, record);
        self.txids_hash ^= Self::txid_position_hash(&txid, position);
        self.bump_content_revision();
    }

    fn sample_recent(&mut self, txid: &Txid, tx: &Transaction) {
        self.recent.insert(0, MempoolRecentTx::from((txid, tx)));
        self.recent.truncate(RECENT_CAP);
    }

    pub fn recent(&self) -> &[MempoolRecentTx] {
        &self.recent
    }

    /// Remove by prefix and return the full record if present. `recent`
    /// is untouched: it's an "added" window, not a live-set mirror.
    pub fn remove_by_prefix(&mut self, prefix: &TxidPrefix) -> Option<TxRecord> {
        let last_position = self.records.len().checked_sub(1)?;
        let last_txid = self.records.last()?.1.entry.txid;
        let (position, _, record) = self.records.swap_remove_full(prefix)?;
        self.txids_hash ^= Self::txid_position_hash(&record.entry.txid, position);
        if position != last_position {
            self.txids_hash ^= Self::txid_position_hash(&last_txid, last_position);
            self.txids_hash ^= Self::txid_position_hash(&last_txid, position);
        }
        self.unresolved.remove(prefix);
        self.histograms.remove(&record);
        self.bump_content_revision();
        Some(record)
    }

    /// Snapshot the round-dollar-eligible histogram that feeds the oracle
    /// blend. Maintained incrementally, so this is `O(NUM_BINS)`, not
    /// `O(live_outputs)`.
    pub fn live_eligible_histogram(&self) -> HistogramRaw {
        self.histograms.eligible()
    }

    /// Snapshot the raw histogram: every live output binned by value with no
    /// payment filtering. Maintained incrementally alongside the eligible one.
    pub fn live_raw_histogram(&self) -> HistogramRaw {
        self.histograms.raw()
    }

    /// Set of prefixes with at least one unfilled prevout. Used by the
    /// prevout filler as a cheap "is there any work?" gate.
    pub fn unresolved(&self) -> &FxHashSet<TxidPrefix> {
        &self.unresolved
    }

    /// Apply resolved prevouts to a tx in place. `fills` is `(vin, prevout)`.
    /// Returns the prevouts actually written (so the caller can fold them
    /// into `AddrTracker`). Updates `unresolved` if fully resolved after
    /// the fill, and refreshes `total_sigop_cost` (P2SH and witness
    /// components depend on prevouts). `entry.vsize` is Core's value from
    /// `MempoolEntryInfo` and is not recomputed here - the sigops shift
    /// belongs to the `Transaction`, not the entry.
    pub fn apply_fills(&mut self, prefix: &TxidPrefix, fills: Vec<(Vin, TxOut)>) -> Vec<TxOut> {
        let Some(record) = self.records.get_mut(prefix) else {
            return Vec::new();
        };
        let applied = Self::write_prevouts(&mut record.tx, fills);
        if applied.is_empty() {
            return applied;
        }
        record.tx.refresh_sigops();
        if record.tx.input.iter().all(|i| i.prevout.is_some()) {
            self.unresolved.remove(prefix);
        }
        self.bump_content_revision();
        applied
    }

    fn bump_content_revision(&mut self) {
        self.content_revision = self.content_revision.wrapping_add(1);
    }

    fn write_prevouts(tx: &mut Transaction, fills: Vec<(Vin, TxOut)>) -> Vec<TxOut> {
        let mut applied = Vec::with_capacity(fills.len());
        for (vin, prevout) in fills {
            if let Some(txin) = tx.input.get_mut(usize::from(vin))
                && txin.prevout.is_none()
            {
                txin.prevout = Some(prevout.clone());
                applied.push(prevout);
            }
        }
        applied
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::{ScriptBuf, hashes::Hash};
    use brk_types::{MempoolEntryInfo, Sats, Timestamp, VSize, Weight};

    use super::*;
    use crate::test_support::{fake_tx, fake_txid, p2wpkh_script};

    fn entry_for(tx: &Transaction, fee: u64, vsize: u64) -> TxEntry {
        let info = MempoolEntryInfo {
            txid: tx.txid,
            vsize: VSize::from(vsize),
            weight: Weight::from(VSize::from(vsize)),
            fee: Sats::from(fee),
            first_seen: Timestamp::from(0u32),
            depends: vec![],
        };
        TxEntry::new(&info, vsize, false)
    }

    fn tx_without_prevouts(seed: u8) -> Transaction {
        fake_tx(seed, &[None, None], &[(p2wpkh_script(1), 1_000)])
    }

    fn tx_with_prevouts(seed: u8) -> Transaction {
        let prev = Some(TxOut::from((p2wpkh_script(2), Sats::from(2_000u64))));
        fake_tx(seed, &[prev], &[(p2wpkh_script(3), 500)])
    }

    fn recompute_txids_hash(store: &TxStore) -> u64 {
        store.txids().enumerate().fold(0, |hash, (position, txid)| {
            hash ^ TxStore::txid_position_hash(txid, position)
        })
    }

    #[test]
    fn insert_records_unresolved_when_prevouts_missing() {
        let mut store = TxStore::default();
        let tx = tx_without_prevouts(1);
        let entry = entry_for(&tx, 100, 100);
        let prefix = entry.txid_prefix();
        store.insert(tx, entry);

        assert!(store.unresolved().contains(&prefix));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn insert_skips_unresolved_when_all_prevouts_present() {
        let mut store = TxStore::default();
        let tx = tx_with_prevouts(2);
        let entry = entry_for(&tx, 200, 150);
        let prefix = entry.txid_prefix();
        store.insert(tx, entry);

        assert!(!store.unresolved().contains(&prefix));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn remove_by_prefix_clears_unresolved_and_returns_record() {
        let mut store = TxStore::default();
        let tx = tx_without_prevouts(3);
        let entry = entry_for(&tx, 300, 200);
        let prefix = entry.txid_prefix();
        store.insert(tx, entry);
        assert!(store.unresolved().contains(&prefix));

        let removed = store.remove_by_prefix(&prefix).expect("record present");
        assert_eq!(removed.entry.txid_prefix(), prefix);
        assert!(!store.unresolved().contains(&prefix));
        assert_eq!(store.len(), 0);
        assert!(store.remove_by_prefix(&prefix).is_none());
    }

    #[test]
    fn swap_remove_preserves_remaining_lookups() {
        let mut store = TxStore::default();
        let mut prefixes = Vec::new();
        for seed in 1..=3 {
            let tx = tx_with_prevouts(seed);
            let entry = entry_for(&tx, 100, 100);
            prefixes.push(entry.txid_prefix());
            store.insert(tx, entry);
        }

        store.remove_by_prefix(&prefixes[1]).expect("middle record");

        assert!(store.record_by_prefix(&prefixes[0]).is_some());
        assert!(store.record_by_prefix(&prefixes[1]).is_none());
        assert!(store.record_by_prefix(&prefixes[2]).is_some());
    }

    #[test]
    fn txids_hash_tracks_inserts_and_swap_removals() {
        let mut store = TxStore::default();
        let mut prefixes = Vec::new();
        assert_eq!(store.txids_hash(), recompute_txids_hash(&store));

        for seed in 4..=6 {
            let tx = tx_with_prevouts(seed);
            let entry = entry_for(&tx, 100, 100);
            prefixes.push(entry.txid_prefix());
            store.insert(tx, entry);
            assert_eq!(store.txids_hash(), recompute_txids_hash(&store));
        }

        store.remove_by_prefix(&prefixes[1]).expect("middle record");
        assert_eq!(store.txids_hash(), recompute_txids_hash(&store));

        store.remove_by_prefix(&prefixes[2]).expect("last record");
        assert_eq!(store.txids_hash(), recompute_txids_hash(&store));

        store.remove_by_prefix(&prefixes[0]).expect("final record");
        assert_eq!(store.txids_hash(), 0);
    }

    #[test]
    fn content_revision_tracks_serialized_body_changes() {
        let mut store = TxStore::default();
        let tx = tx_without_prevouts(7);
        let entry = entry_for(&tx, 100, 100);
        let prefix = entry.txid_prefix();
        assert_eq!(store.content_revision(), 0);

        store.insert(tx, entry);
        assert_eq!(store.content_revision(), 1);

        assert!(store.apply_fills(&prefix, Vec::new()).is_empty());
        assert_eq!(store.content_revision(), 1);

        let prevout = TxOut::from((p2wpkh_script(8), Sats::from(2_000u64)));
        let applied = store.apply_fills(&prefix, vec![(Vin::from(0usize), prevout)]);
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].value, Sats::from(2_000u64));
        assert_eq!(store.content_revision(), 2);

        assert!(
            store
                .apply_fills(
                    &prefix,
                    vec![(
                        Vin::from(0usize),
                        TxOut::from((p2wpkh_script(9), Sats::from(3_000u64))),
                    )],
                )
                .is_empty()
        );
        assert_eq!(store.content_revision(), 2);

        store.remove_by_prefix(&prefix).expect("stored record");
        assert_eq!(store.content_revision(), 3);
        assert!(store.remove_by_prefix(&prefix).is_none());
        assert_eq!(store.content_revision(), 3);
    }

    #[test]
    fn full_txid_lookups_reject_prefix_collisions() {
        let mut store = TxStore::default();
        let tx = tx_with_prevouts(9);
        let stored_txid = tx.txid;
        let entry = entry_for(&tx, 100, 100);
        store.insert(tx, entry);

        let mut bytes = bitcoin::Txid::from(&stored_txid).to_byte_array();
        bytes[8] ^= 1;
        let colliding_txid = Txid::from(bitcoin::Txid::from_byte_array(bytes));
        assert_eq!(
            TxidPrefix::from(&stored_txid),
            TxidPrefix::from(&colliding_txid)
        );

        assert!(store.contains(&stored_txid));
        assert!(store.get(&stored_txid).is_some());
        assert!(store.entry(&stored_txid).is_some());
        assert!(!store.contains(&colliding_txid));
        assert!(store.get(&colliding_txid).is_none());
        assert!(store.entry(&colliding_txid).is_none());
    }

    #[test]
    fn apply_fills_writes_only_missing_inputs_and_refreshes_sigops() {
        let mut store = TxStore::default();
        let prev_present = TxOut::from((p2wpkh_script(4), Sats::from(7_000u64)));
        let tx = fake_tx(
            4,
            &[None, Some(prev_present.clone())],
            &[(p2wpkh_script(5), 1_000)],
        );
        let entry = entry_for(&tx, 400, 250);
        let prefix = entry.txid_prefix();
        store.insert(tx, entry);
        assert!(store.unresolved().contains(&prefix));

        let new_prevout = TxOut::from((p2wpkh_script(6), Sats::from(9_000u64)));
        let overwrite_attempt = TxOut::from((p2wpkh_script(99), Sats::from(1u64)));
        let applied = store.apply_fills(
            &prefix,
            vec![
                (Vin::from(0u32), new_prevout.clone()),
                (Vin::from(1u32), overwrite_attempt),
            ],
        );

        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].value, new_prevout.value);

        let record = store.record_by_prefix(&prefix).expect("record present");
        assert_eq!(
            record.tx.input[0].prevout.as_ref().unwrap().value,
            new_prevout.value
        );
        assert_eq!(
            record.tx.input[1].prevout.as_ref().unwrap().value,
            prev_present.value
        );
        assert!(!store.unresolved().contains(&prefix));
    }

    #[test]
    fn apply_fills_unknown_prefix_is_noop() {
        let mut store = TxStore::default();
        let stray_prefix = TxidPrefix::from(&fake_txid(0xFF));
        let applied = store.apply_fills(
            &stray_prefix,
            vec![(
                Vin::from(0u32),
                TxOut::from((ScriptBuf::new(), Sats::from(1u64))),
            )],
        );
        assert!(applied.is_empty());
    }

    #[test]
    fn apply_fills_partial_keeps_unresolved() {
        let mut store = TxStore::default();
        let tx = tx_without_prevouts(5);
        let entry = entry_for(&tx, 500, 300);
        let prefix = entry.txid_prefix();
        store.insert(tx, entry);

        let one = TxOut::from((p2wpkh_script(7), Sats::from(3_000u64)));
        let applied = store.apply_fills(&prefix, vec![(Vin::from(0u32), one)]);
        assert_eq!(applied.len(), 1);
        assert!(
            store.unresolved().contains(&prefix),
            "input 1 still has None prevout"
        );
    }

    #[test]
    fn recent_is_capped_and_newest_first() {
        let mut store = TxStore::default();
        for i in 0..(RECENT_CAP as u8 + 5) {
            let tx = tx_with_prevouts(i + 10);
            let entry = entry_for(&tx, 100, 100);
            store.insert(tx, entry);
        }
        assert_eq!(store.recent().len(), RECENT_CAP);
        let newest = store.recent().first().expect("at least one");
        let last_inserted_txid = fake_txid(RECENT_CAP as u8 + 5 + 10 - 1);
        assert_eq!(newest.txid, last_inserted_txid);
    }

    #[test]
    fn live_histogram_total_tracks_inserts_and_removes() {
        let mut store = TxStore::default();
        let tx_a = fake_tx(
            20,
            &[Some(TxOut::from((p2wpkh_script(8), Sats::from(1_234u64))))],
            &[(p2wpkh_script(9), 2_345), (p2wpkh_script(10), 3_456)],
        );
        let tx_b = fake_tx(
            21,
            &[Some(TxOut::from((p2wpkh_script(11), Sats::from(4_567u64))))],
            &[(p2wpkh_script(12), 7_891)],
        );
        let entry_a = entry_for(&tx_a, 100, 100);
        let entry_b = entry_for(&tx_b, 100, 100);
        let prefix_a = entry_a.txid_prefix();
        store.insert(tx_a, entry_a);
        store.insert(tx_b, entry_b);

        let total_after_both: u32 = store.live_eligible_histogram().iter().sum();
        assert_eq!(total_after_both, 3, "two outputs + one output");

        store.remove_by_prefix(&prefix_a);
        let total_after_remove: u32 = store.live_eligible_histogram().iter().sum();
        assert_eq!(total_after_remove, 1);
    }

    #[test]
    fn raw_histogram_bins_outputs_the_eligible_filter_drops() {
        let mut store = TxStore::default();
        // 2_345 sats is a round-dollar-eligible payment; 100_000_000 sats (1 BTC)
        // is a round-BTC value the eligible filter drops but raw still bins.
        let tx = fake_tx(
            30,
            &[Some(TxOut::from((p2wpkh_script(1), Sats::from(50_000u64))))],
            &[(p2wpkh_script(2), 2_345), (p2wpkh_script(3), 100_000_000)],
        );
        let entry = entry_for(&tx, 100, 100);
        let prefix = entry.txid_prefix();
        store.insert(tx, entry);

        assert_eq!(
            store.live_eligible_histogram().iter().sum::<u32>(),
            1,
            "round-BTC output filtered out of the eligible histogram"
        );
        assert_eq!(
            store.live_raw_histogram().iter().sum::<u32>(),
            2,
            "raw histogram bins every output"
        );

        store.remove_by_prefix(&prefix);
        assert_eq!(store.live_eligible_histogram().iter().sum::<u32>(), 0);
        assert_eq!(store.live_raw_histogram().iter().sum::<u32>(), 0);
    }
}
