use crate::Mempool;
use brk_types::{BlockTemplateDiffEntry, FeeRate, Sats, TxOut, TxidPrefix, Vin};
use serde_json::to_vec;

use super::*;
use crate::{
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, p2wpkh_script},
};

fn insert_tx(mempool: &mut Mempool, seed: u8, fee: u64, vsize: u64) -> Txid {
    let tx = fake_tx(seed, &[None], &[(p2wpkh_script(seed + 1), 1_234)]);
    let txid = tx.txid;
    let info = fake_entry_info(txid, fee, vsize);
    let entry = TxEntry::new(&info, vsize, false);
    let state = mempool.test_state_mut();
    state.txs.insert(tx, entry);
    txid
}

#[test]
fn block_template_source_tracks_snapshot_and_body_changes() {
    let mut mempool = Mempool::for_test();
    let txid = insert_tx(&mut mempool, 0xA7, 1_234, 100);
    mempool.test_tick(&[txid], FeeRate::new(1.0));
    let initial = mempool.published().block_template_source();

    let prefix = TxidPrefix::from(&txid);
    let prevout = TxOut::from((p2wpkh_script(0xA8), Sats::from(2_000u64)));
    mempool
        .test_state_mut()
        .txs
        .apply_fills(&prefix, vec![(Vin::from(0usize), prevout)]);
    let after_body_change = mempool.published().block_template_source();
    assert!(
        initial == after_body_change,
        "unpublished fills cannot change a published template"
    );

    mempool.test_tick(&[txid], FeeRate::new(2.0));
    let after_snapshot_change = mempool.published().block_template_source();
    assert!(after_body_change != after_snapshot_change);
}

#[test]
fn block_template_diff_round_trip_reconstructs_t1_from_t0() {
    // T0: pool has two txs, both in gbt -> block 0.
    let mut mempool = Mempool::for_test();
    let txid_a = insert_tx(&mut mempool, 0xA1, 1_111, 100);
    let txid_b = insert_tx(&mut mempool, 0xA2, 2_222, 100);
    mempool.test_tick(&[txid_a, txid_b], FeeRate::new(1.0));
    let t0 = mempool.published().block_template_source().build().unwrap();

    // T1: add a third tx, advance gbt. block_template_diff(t0.hash) must
    // be reconstructible into the new block 0 ordering by combining the
    // retained prior-indexed bodies from T0 with the New bodies inline.
    let txid_c = insert_tx(&mut mempool, 0xA3, 3_333, 100);
    mempool.test_tick(&[txid_a, txid_b, txid_c], FeeRate::new(1.0));
    let t1 = mempool.published().block_template_source().build().unwrap();

    let diff = mempool
        .published()
        .resolve_block_template_diff(t0.hash)
        .map(|resolved| resolved.build())
        .transpose()
        .unwrap()
        .expect("t0 is still in history");
    assert_eq!(diff.since, t0.hash);
    assert_eq!(diff.hash, t1.hash);

    let mut reconstructed = Vec::with_capacity(diff.order.len());
    for entry in &diff.order {
        match entry {
            BlockTemplateDiffEntry::Retained(idx) => {
                reconstructed.push(t0.transactions[*idx as usize].clone());
            }
            BlockTemplateDiffEntry::New(tx) => reconstructed.push(tx.clone()),
        }
    }
    let expected: Vec<_> = t1.transactions.iter().map(|tx| tx.txid).collect();
    let got: Vec<_> = reconstructed.iter().map(|tx| tx.txid).collect();
    assert_eq!(got, expected, "diff round-trips back into T1 ordering");
    assert!(diff.removed.is_empty());
}

#[test]
fn block_template_diff_preserves_reordering_and_prior_removal_order() {
    let mut mempool = Mempool::for_test();
    let a = insert_tx(&mut mempool, 30, 100, 100);
    let b = insert_tx(&mut mempool, 31, 100, 100);
    let c = insert_tx(&mut mempool, 32, 100, 100);
    let d = insert_tx(&mut mempool, 33, 100, 100);
    let added = insert_tx(&mut mempool, 34, 100, 100);
    mempool.test_tick(&[a, b, c, d], FeeRate::new(1.0));
    let before = mempool.published().next_block_hash().unwrap();

    mempool.test_tick(&[d, added, b], FeeRate::new(1.0));
    let diff = mempool
        .published()
        .resolve_block_template_diff(before)
        .map(|resolved| resolved.build())
        .transpose()
        .unwrap()
        .unwrap();
    assert_eq!(diff.removed, [a, c]);
    assert_eq!(diff.order.len(), 3);
    assert!(matches!(diff.order[0], BlockTemplateDiffEntry::Retained(3)));
    assert!(matches!(&diff.order[1], BlockTemplateDiffEntry::New(tx) if tx.txid == added));
    assert!(matches!(diff.order[2], BlockTemplateDiffEntry::Retained(1)));

    mempool.test_tick(&[], FeeRate::new(1.0));
    let empty = mempool
        .published()
        .resolve_block_template_diff(before)
        .map(|resolved| resolved.build())
        .transpose()
        .unwrap()
        .unwrap();
    assert!(empty.order.is_empty());
    assert_eq!(empty.removed, [a, b, c, d]);
}

#[test]
fn block_template_diff_unknown_since_returns_none() {
    let mut mempool = Mempool::for_test();
    mempool.test_tick(&[], FeeRate::new(1.0));
    let bogus = NextBlockHash::new(0xDEAD_BEEF);
    assert!(
        mempool
            .published()
            .resolve_block_template_diff(bogus)
            .is_none()
    );
}

#[test]
fn resolved_template_and_diff_keep_the_validated_publication_after_history_eviction() {
    let mut mempool = Mempool::for_test();
    let first = insert_tx(&mut mempool, 0xB0, 1_000, 100);
    let mut txids = vec![first];
    mempool.test_tick(&txids, FeeRate::new(1.0));
    let since = mempool.published().next_block_hash().unwrap();
    let source = mempool.published().block_template_source();
    let resolved = mempool
        .published()
        .resolve_block_template_diff(since)
        .expect("initial template in history");

    for seed in 0xB1..=0xBB {
        txids.push(insert_tx(&mut mempool, seed, 1_000, 100));
        mempool.test_tick(&txids, FeeRate::new(1.0));
    }
    assert!(
        mempool
            .published()
            .resolve_block_template_diff(since)
            .is_none()
    );

    let diff = resolved.build().unwrap();
    assert_eq!(diff.since, since);
    assert_eq!(diff.hash, since);
    assert_eq!(diff.order.len(), 1);
    let template = source.build().unwrap();
    assert_eq!(template.hash, since);
    assert_eq!(template.transactions.len(), 1);
    assert_eq!(template.transactions[0].txid, first);
}

#[test]
fn body_fills_publish_a_new_identity_and_diff_reconstructs_every_field() {
    let mut mempool = Mempool::for_test();
    let changed = insert_tx(&mut mempool, 10, 100, 100);
    let stable = insert_tx(&mut mempool, 11, 100, 100);
    let ids = [changed, stable];
    mempool.test_tick(&ids, FeeRate::new(1.0));
    let before = mempool.published().block_template_source().build().unwrap();
    let published = mempool.published().snapshot();
    mempool.test_state_mut().txs.apply_fills(
        &TxidPrefix::from(changed),
        vec![(
            Vin::from(0usize),
            TxOut::from((p2wpkh_script(12), Sats::from(2_000u64))),
        )],
    );
    assert!(
        published.template_transactions()[0].input[0]
            .prevout
            .is_none()
    );
    assert_eq!(
        to_vec(&mempool.published().block_template_source().build().unwrap()).unwrap(),
        to_vec(&before).unwrap()
    );
    // Body changes alone must rebuild: no GBT, membership or fee-floor change.
    mempool
        .rebuilder
        .tick(&mempool.state, &ids, FeeRate::new(1.0));
    mempool.publish_observation(Default::default(), true);
    let after = mempool.published().block_template_source().build().unwrap();
    assert_ne!(before.hash, after.hash);
    assert!(after.transactions[0].input[0].prevout.is_some());
    assert!(Arc::ptr_eq(
        &published.template_transactions()[1],
        &mempool.published().snapshot().template_transactions()[1]
    ));
    let diff = mempool
        .published()
        .resolve_block_template_diff(before.hash)
        .map(|resolved| resolved.build())
        .transpose()
        .unwrap()
        .unwrap();
    assert!(diff.removed.is_empty());
    assert!(matches!(&diff.order[0], BlockTemplateDiffEntry::New(_)));
    assert!(matches!(
        &diff.order[1],
        BlockTemplateDiffEntry::Retained(1)
    ));
    let reconstructed: Vec<_> = diff
        .order
        .into_iter()
        .map(|entry| match entry {
            BlockTemplateDiffEntry::Retained(index) => before.transactions[index as usize].clone(),
            BlockTemplateDiffEntry::New(tx) => tx,
        })
        .collect();
    assert_eq!(
        to_vec(&reconstructed).unwrap(),
        to_vec(&after.transactions).unwrap()
    );
}

#[test]
fn published_bodies_survive_removal_and_incomplete_selection_is_not_served() {
    let mut mempool = Mempool::for_test();
    let txid = insert_tx(&mut mempool, 20, 100, 100);
    mempool.test_tick(&[txid], FeeRate::new(1.0));
    let before = mempool.published().block_template_source().build().unwrap();
    mempool
        .test_state_mut()
        .txs
        .remove_by_prefix(&TxidPrefix::from(txid));
    assert_eq!(
        to_vec(&mempool.published().block_template_source().build().unwrap()).unwrap(),
        to_vec(&before).unwrap()
    );
    mempool.test_tick(&[txid], FeeRate::new(1.0));
    assert_eq!(
        to_vec(&mempool.published().block_template_source().build().unwrap()).unwrap(),
        to_vec(&before).unwrap()
    );
    let diff = mempool
        .published()
        .resolve_block_template_diff(before.hash)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(diff.hash, before.hash);
}
