use brk_types::{Sats, VSize};

use super::*;
use crate::{
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, p2wpkh_script},
};

fn state_with(seeds: &[u8]) -> (State, Vec<Txid>) {
    let mut state = State::default();
    let mut txids = Vec::with_capacity(seeds.len());
    for &seed in seeds {
        let tx = fake_tx(seed, &[], &[(p2wpkh_script(seed), 1_000)]);
        let txid = tx.txid;
        let entry = TxEntry::new(&fake_entry_info(txid, 100, 100), 100, false);
        state.txs.insert(tx, entry);
        txids.push(txid);
    }
    (state, txids)
}

fn min_fee(sats: u64) -> FeeRate {
    FeeRate::from((Sats::from(sats), VSize::from(1_000u64)))
}

#[test]
fn content_changes_rebuild_even_when_other_reuse_inputs_match() {
    let mut rebuilder = Rebuilder::default();
    let (mut state, txids) = state_with(&[1]);
    rebuilder.tick(&state, &txids, min_fee(1));
    let before = rebuilder.snapshot();

    let tx = fake_tx(2, &[], &[(p2wpkh_script(2), 2_000)]);
    let entry = TxEntry::new(&fake_entry_info(tx.txid, 200, 100), 100, false);
    state.txs.insert(tx, entry);
    rebuilder.tick(&state, &txids, min_fee(1));

    let after = rebuilder.snapshot();
    assert_eq!(rebuilder.rebuild_count(), 2);
    assert!(!Arc::ptr_eq(&before, &after));
    assert_eq!(after.txs.len(), 2);
    assert_eq!(after.content_revision(), state.txs.content_revision());

    rebuilder.tick(&state, &txids, min_fee(1));
    assert!(Arc::ptr_eq(&after, &rebuilder.snapshot()));
    assert_eq!(rebuilder.rebuild_count(), 2);
}

#[test]
fn reordered_template_rebuilds() {
    let mut rebuilder = Rebuilder::default();
    let (state, mut txids) = state_with(&[1, 2]);
    rebuilder.tick(&state, &txids, min_fee(1));
    txids.reverse();

    rebuilder.tick(&state, &txids, min_fee(1));

    assert_eq!(rebuilder.rebuild_count(), 2);
}

#[test]
fn changed_min_fee_rebuilds() {
    let mut rebuilder = Rebuilder::default();
    let (state, txids) = state_with(&[1]);
    rebuilder.tick(&state, &txids, min_fee(1));

    rebuilder.tick(&state, &txids, min_fee(2));

    assert_eq!(rebuilder.rebuild_count(), 2);
}

#[test]
fn removal_outside_template_rebuilds() {
    let mut rebuilder = Rebuilder::default();
    let (mut state, txids) = state_with(&[1, 2]);
    let template = &txids[..1];
    rebuilder.tick(&state, template, min_fee(1));

    state.txs.remove_by_prefix(&txids[1].into()).unwrap();
    rebuilder.tick(&state, template, min_fee(1));

    assert_eq!(rebuilder.rebuild_count(), 2);
    assert_eq!(rebuilder.snapshot().txs.len(), 1);
}
