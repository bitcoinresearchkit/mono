use bitcoin::{OutPoint as BitcoinOutPoint, Txid as BitcoinTxid, hashes::Hash};
use brk_types::{FeeRate, Sats, TxOut, VSize};

use super::*;
use crate::{
    TxRemoval,
    state::TxEntry,
    steps::preparer::TxAddition,
    test_support::{fake_bitcoin_tx, fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
};

fn empty_state() -> State {
    State::default()
}

fn seed_known(state: &mut State, txid: Txid) {
    let tx = fake_tx(0xA0, &[None], &[(p2wpkh_script(50), 5_000)]);
    let mut altered = tx;
    altered.txid = txid;
    for input in altered.input.iter_mut() {
        input.prevout = Some(TxOut::from((p2wpkh_script(51), Sats::from(1_000u64))));
    }
    let info = fake_entry_info(txid, 1_000, 100);
    let entry = TxEntry::new(&info, 100, false);
    state.txs.insert(altered, entry);
}

fn seed_graveyard(state: &mut State, txid: Txid) {
    let tx = fake_tx(0xB0, &[None], &[(p2wpkh_script(60), 5_000)]);
    let mut altered = tx;
    altered.txid = txid;
    let info = fake_entry_info(txid, 500, 100);
    let entry = TxEntry::new(&info, 100, false);
    let rate = FeeRate::from((Sats::from(500u64), VSize::from(100u64)));
    state
        .graveyard
        .bury(altered, entry, rate, TxRemoval::Vanished);
}

#[test]
fn classify_addition_skips_already_known() {
    let mut state = empty_state();
    let known_txid = fake_txid(0x10);
    seed_known(&mut state, known_txid);

    let info = fake_entry_info(known_txid, 100, 100);
    let mut new_txs: FxHashMap<Txid, BitcoinTransaction> = FxHashMap::default();
    new_txs.insert(
        known_txid,
        fake_bitcoin_tx(0x11, &[(p2wpkh_script(7), 1_234)]),
    );

    let pulled = prepare(&[known_txid], vec![info], new_txs, &state);
    assert!(pulled.added.is_empty(), "known tx must be filtered out");
    assert!(pulled.removed.is_empty(), "still live, nothing removed");
}

#[test]
fn classify_addition_emits_revived_for_graveyard_hit() {
    let mut state = empty_state();
    let txid = fake_txid(0x20);
    seed_graveyard(&mut state, txid);

    let info = fake_entry_info(txid, 100, 100);
    let pulled = prepare(&[txid], vec![info], FxHashMap::default(), &state);

    assert_eq!(pulled.added.len(), 1);
    assert!(matches!(pulled.added[0], TxAddition::Revived { .. }));
}

#[test]
fn classify_addition_emits_fresh_with_raw_payload() {
    let state = empty_state();
    let txid = fake_txid(0x30);
    // Make the bitcoin tx hash to `txid`: we instead key `new_txs`
    // by the synthetic txid, since classify_addition keys lookup by
    // info.txid, not by tx.compute_txid().
    let info = fake_entry_info(txid, 200, 120);
    let raw = fake_bitcoin_tx(0x31, &[(p2wpkh_script(8), 2_345)]);
    let mut new_txs: FxHashMap<Txid, BitcoinTransaction> = FxHashMap::default();
    new_txs.insert(txid, raw);

    let pulled = prepare(&[txid], vec![info], new_txs, &state);
    assert_eq!(pulled.added.len(), 1);
    assert!(matches!(pulled.added[0], TxAddition::Fresh { .. }));
}

#[test]
fn classify_addition_drops_entry_with_no_raw_and_no_graveyard() {
    let state = empty_state();
    let txid = fake_txid(0x40);
    let info = fake_entry_info(txid, 100, 100);

    let pulled = prepare(&[txid], vec![info], FxHashMap::default(), &state);
    assert!(pulled.added.is_empty(), "no payload, no tomb -> filtered");
}

#[test]
fn classify_removal_marks_replaced_when_outpoint_is_spent_by_new_tx() {
    let mut state = empty_state();
    // Loser: spends (parent, vout=0). We arrange the new fresh tx
    // to spend the same outpoint.
    let parent_txid = fake_txid(0x50);
    let loser_txid = fake_txid(0x51);
    let replacer_txid = fake_txid(0x52);
    {
        let prev = Some(TxOut::from((p2wpkh_script(80), Sats::from(10_000u64))));
        let mut tx = fake_tx(0x51, &[prev], &[(p2wpkh_script(81), 5_000)]);
        tx.txid = loser_txid;
        tx.input[0].txid = parent_txid;
        tx.input[0].vout = Vout::ZERO;
        let info = fake_entry_info(loser_txid, 100, 100);
        let entry = TxEntry::new(&info, 100, false);
        state.txs.insert(tx, entry);
    }

    let info = fake_entry_info(replacer_txid, 200, 120);
    let mut new_txs: FxHashMap<Txid, BitcoinTransaction> = FxHashMap::default();
    let mut raw = fake_bitcoin_tx(0x52, &[(p2wpkh_script(82), 4_321)]);
    raw.input[0].previous_output = BitcoinOutPoint {
        txid: BitcoinTxid::from_byte_array({
            let mut b = [0u8; 32];
            b[0] = 0x50;
            b
        }),
        vout: 0,
    };
    new_txs.insert(replacer_txid, raw);

    let pulled = prepare(&[replacer_txid], vec![info], new_txs, &state);
    assert_eq!(pulled.removed.len(), 1);
    let (_, reason) = pulled.removed[0];
    match reason {
        TxRemoval::Replaced { by } => assert_eq!(by, replacer_txid),
        TxRemoval::Vanished => panic!("expected Replaced, got Vanished"),
    }
}

#[test]
fn classify_removal_marks_vanished_when_no_new_tx_spends_outpoint() {
    let mut state = empty_state();
    let gone_txid = fake_txid(0x60);
    {
        let prev = Some(TxOut::from((p2wpkh_script(90), Sats::from(10_000u64))));
        let mut tx = fake_tx(0x60, &[prev], &[(p2wpkh_script(91), 6_000)]);
        tx.txid = gone_txid;
        tx.input[0].txid = fake_txid(0xAA);
        let info = fake_entry_info(gone_txid, 100, 100);
        let entry = TxEntry::new(&info, 100, false);
        state.txs.insert(tx, entry);
    }

    // No live txids in this cycle, no replacers staged.
    let pulled = prepare(&[], vec![], FxHashMap::default(), &state);
    assert_eq!(pulled.removed.len(), 1);
    assert!(matches!(pulled.removed[0].1, TxRemoval::Vanished));
}

#[test]
fn fresh_resolves_prevout_from_same_cycle_mempool_parent() {
    // Same-cycle ordering: parent inserted first, then child whose
    // input points at parent.vout=0. We exercise the path by
    // putting the parent into the live store via a Fresh add, then
    // a second Preparer call where the child's `info` references
    // the parent's outpoint.
    let mut state = empty_state();
    let parent_txid = fake_txid(0x70);
    let child_txid = fake_txid(0x71);

    // Stage parent directly in the live store so resolve_prevout
    // sees it when the child is decoded.
    {
        let mut parent = fake_tx(0x70, &[], &[(p2wpkh_script(100), 7_777)]);
        parent.txid = parent_txid;
        parent.input.clear();
        let info = fake_entry_info(parent_txid, 100, 80);
        let entry = TxEntry::new(&info, 80, false);
        state.txs.insert(parent, entry);
    }

    let info = fake_entry_info(child_txid, 200, 120);
    let mut raw = fake_bitcoin_tx(0x70, &[(p2wpkh_script(101), 6_000)]);
    raw.input[0].previous_output = BitcoinOutPoint {
        txid: BitcoinTxid::from_byte_array({
            let mut b = [0u8; 32];
            b[0] = 0x70;
            b
        }),
        vout: 0,
    };
    let mut new_txs: FxHashMap<Txid, BitcoinTransaction> = FxHashMap::default();
    new_txs.insert(child_txid, raw);

    let pulled = prepare(&[parent_txid, child_txid], vec![info], new_txs, &state);
    let TxAddition::Fresh { tx, .. } = &pulled.added[0] else {
        panic!("expected Fresh classification");
    };
    let prevout = tx.input[0]
        .prevout
        .as_ref()
        .expect("parent in same-cycle pool must resolve");
    assert_eq!(prevout.value, Sats::from(7_777u64));
    // No removal: parent + child both in live set.
    assert!(pulled.removed.is_empty());
}
