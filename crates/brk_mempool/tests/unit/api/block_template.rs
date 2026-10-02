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
fn block_template_diff_preserves_reordering_and_prior_removal_order() {
    let mut mempool = Mempool::for_test();
    let a = insert_tx(&mut mempool, 30, 100, 100);
    let b = insert_tx(&mut mempool, 31, 100, 100);
    let c = insert_tx(&mut mempool, 32, 100, 100);
    let d = insert_tx(&mut mempool, 33, 100, 100);
    let added = insert_tx(&mut mempool, 34, 100, 100);
    mempool.test_tick(&[a, b, c, d], FeeRate::new(1.0));
    let before = mempool.published().next_block_hash().unwrap();

    let mut previous = before;
    for ids in [&[d, c, b, a][..], &[d, c, b][..]] {
        mempool.test_tick(ids, FeeRate::new(1.0));
        let hash = mempool.published().next_block_hash().unwrap();
        assert_ne!(hash, previous);
        previous = hash;
    }

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
fn body_fills_publish_a_new_identity_and_diff_reconstructs_every_field() {
    let mut mempool = Mempool::for_test();
    let changed = insert_tx(&mut mempool, 10, 100, 100);
    let stable = insert_tx(&mut mempool, 11, 100, 100);
    let ids = [changed, stable];
    mempool.test_tick(&ids, FeeRate::new(1.0));
    let source = mempool.published().block_template_source();
    let before = source.clone().build().unwrap();
    let published = mempool.published().snapshot();
    mempool.test_state_mut().txs.apply_fills(
        &TxidPrefix::from(changed),
        vec![(
            Vin::from(0usize),
            TxOut::from((p2wpkh_script(12), Sats::from(2_000u64))),
        )],
    );
    assert!(source == mempool.published().block_template_source());
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
    assert!(source != mempool.published().block_template_source());
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
