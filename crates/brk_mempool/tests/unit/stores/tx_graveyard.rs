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
        .replacement_candidates_recent_first()
        .flatten()
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
