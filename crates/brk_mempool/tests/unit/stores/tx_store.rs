use bitcoin::{Txid as BitcoinTxid, hashes::Hash};
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
    assert!(store.record_by_prefix(&prefixes[0]).is_some());
    assert!(store.record_by_prefix(&prefixes[1]).is_none());
    assert!(store.record_by_prefix(&prefixes[2]).is_some());
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
    assert_eq!(store.len(), 1);
    assert!(store.unresolved().contains(&prefix));
    assert_eq!(store.content_revision(), 1);

    assert!(store.apply_fills(&prefix, Vec::new()).is_empty());
    assert_eq!(store.content_revision(), 1);

    let prevout = TxOut::from((p2wpkh_script(8), Sats::from(2_000u64)));
    let applied = store.apply_fills(&prefix, vec![(Vin::from(0usize), prevout.clone())]);
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].value, Sats::from(2_000u64));
    assert_eq!(store.content_revision(), 2);
    assert!(store.unresolved().contains(&prefix));

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

    let removed = store.remove_by_prefix(&prefix).expect("stored record");
    assert_eq!(removed.entry.txid_prefix(), prefix);
    assert_eq!(store.len(), 0);
    assert!(!store.unresolved().contains(&prefix));
    assert_eq!(store.content_revision(), 3);
    assert!(store.remove_by_prefix(&prefix).is_none());
    assert!(
        store
            .apply_fills(&prefix, vec![(Vin::from(0usize), prevout)])
            .is_empty()
    );
    assert_eq!(store.content_revision(), 3);
}

#[test]
fn full_txid_lookups_reject_prefix_collisions() {
    let mut store = TxStore::default();
    let tx = tx_with_prevouts(9);
    let stored_txid = tx.txid;
    let entry = entry_for(&tx, 100, 100);
    store.insert(tx, entry);
    assert_eq!(store.len(), 1);
    assert!(store.unresolved().is_empty());

    let mut bytes = BitcoinTxid::from(&stored_txid).to_byte_array();
    bytes[8] ^= 1;
    let colliding_txid = Txid::from(BitcoinTxid::from_byte_array(bytes));
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
