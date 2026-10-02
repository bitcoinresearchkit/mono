use brk_types::{Sats, VSize};

use super::*;
use crate::{
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, p2wpkh_script},
};

fn min_fee(sats: u64) -> FeeRate {
    FeeRate::from((Sats::from(sats), VSize::from(1_000u64)))
}

#[test]
fn reuse_requires_matching_content_and_fee() {
    let mut rebuilder = Rebuilder::default();
    let mut state = State::default();
    let tx = fake_tx(1, &[], &[(p2wpkh_script(1), 1_000)]);
    let txids = [tx.txid];
    let entry = TxEntry::new(&fake_entry_info(tx.txid, 100, 100), 100, false);
    state.txs.insert(tx, entry);
    rebuilder.tick(&state, &txids, min_fee(1));
    let before = rebuilder.snapshot();

    let tx = fake_tx(2, &[], &[(p2wpkh_script(2), 2_000)]);
    let outside_template = tx.txid;
    let entry = TxEntry::new(&fake_entry_info(outside_template, 200, 100), 100, false);
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

    rebuilder.tick(&state, &txids, min_fee(2));
    let fee_changed = rebuilder.snapshot();
    assert!(!Arc::ptr_eq(&after, &fee_changed));
    assert_eq!(fee_changed.min_fee, min_fee(2));
    assert_eq!(rebuilder.rebuild_count(), 3);

    state
        .txs
        .remove_by_prefix(&outside_template.into())
        .unwrap();
    rebuilder.tick(&state, &txids, min_fee(2));
    let removed = rebuilder.snapshot();
    assert!(!Arc::ptr_eq(&fee_changed, &removed));
    assert_eq!(rebuilder.rebuild_count(), 4);
    assert_eq!(removed.txs.len(), 1);
    assert_eq!(removed.content_revision(), state.txs.content_revision());
}
