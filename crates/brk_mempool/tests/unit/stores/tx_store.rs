use brk_types::{MempoolEntryInfo, Sats, Timestamp, VSize, Weight};

use super::*;
use crate::test_support::{fake_tx, p2wpkh_script};

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

