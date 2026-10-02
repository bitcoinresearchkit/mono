use bitcoin::OutPoint as BitcoinOutPoint;
use brk_types::{FeeRate, Sats, TxOut, VSize};

use super::*;
use crate::{
    TxRemoval,
    state::TxEntry,
    steps::preparer::TxAddition,
    test_support::{fake_bitcoin_tx, fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
};

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
fn mixed_additions_skip_known_and_missing_bodies_and_resolve_live_parents() {
    let mut state = State::default();
    let known = fake_txid(0x10);
    let revived = fake_txid(0x20);
    let fresh = fake_txid(0x30);
    let missing = fake_txid(0x40);
    seed_known(&mut state, known);
    seed_graveyard(&mut state, revived);

    let mut raw = fake_bitcoin_tx(0x31, &[(p2wpkh_script(8), 2_345)]);
    raw.input[0].previous_output = BitcoinOutPoint {
        txid: known.into(),
        vout: 0,
    };
    let new_txs = FxHashMap::from_iter([
        (known, fake_bitcoin_tx(0x11, &[(p2wpkh_script(7), 1_234)])),
        (fresh, raw),
    ]);
    let live = [known, revived, fresh, missing];
    let entries = live
        .iter()
        .map(|&txid| fake_entry_info(txid, 200, 120))
        .collect();
    let pulled = prepare(&live, entries, new_txs, &state);

    assert!(pulled.removed.is_empty());
    assert_eq!(pulled.added.len(), 2);
    let TxAddition::Revived { entry } = &pulled.added[0] else {
        panic!("expected revived transaction");
    };
    assert_eq!(entry.txid, revived);
    let TxAddition::Fresh { tx, .. } = &pulled.added[1] else {
        panic!("expected fresh transaction");
    };
    assert_eq!(tx.txid, fresh);
    assert_eq!(
        tx.input[0].prevout.as_ref().unwrap().value,
        Sats::new(5_000)
    );
}

#[test]
fn removals_distinguish_conflicting_outpoints_from_vanished_transactions() {
    let mut state = State::default();
    let parent = fake_txid(0x50);
    let loser = fake_txid(0x51);
    let gone = fake_txid(0x60);
    let replacer = fake_txid(0x52);
    for (txid, origin) in [(loser, parent), (gone, fake_txid(0xAA))] {
        let prev = Some(TxOut::from((p2wpkh_script(80), Sats::new(10_000))));
        let mut tx = fake_tx(0x51, &[prev], &[(p2wpkh_script(81), 5_000)]);
        tx.txid = txid;
        tx.input[0].txid = origin;
        tx.input[0].vout = Vout::ZERO;
        let entry = TxEntry::new(&fake_entry_info(txid, 100, 100), 100, false);
        state.txs.insert(tx, entry);
    }
    let mut raw = fake_bitcoin_tx(0x52, &[(p2wpkh_script(82), 4_321)]);
    raw.input[0].previous_output = BitcoinOutPoint {
        txid: parent.into(),
        vout: 0,
    };
    let pulled = prepare(
        &[replacer],
        vec![fake_entry_info(replacer, 200, 120)],
        FxHashMap::from_iter([(replacer, raw)]),
        &state,
    );
    assert_eq!(pulled.removed.len(), 2);
    assert_eq!(pulled.removed[0].0, TxidPrefix::from(loser));
    assert!(matches!(pulled.removed[0].1, TxRemoval::Replaced { by } if by == replacer));
    assert_eq!(pulled.removed[1].0, TxidPrefix::from(gone));
    assert!(matches!(pulled.removed[1].1, TxRemoval::Vanished));
}
