use crate::Mempool;
use brk_types::{FeeRate, Sats, TxOut, Txid};

use super::*;
use crate::{
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, p2wpkh_script},
};

/// Insert a tx, optionally declaring parent dependencies for the
/// snapshot builder's adjacency wire-up.
fn insert_with_depends(
    mempool: &mut Mempool,
    seed: u8,
    fee: u64,
    vsize: u64,
    parents: &[Txid],
) -> Txid {
    let prevout = TxOut::from((p2wpkh_script(seed), Sats::from(20_000u64)));
    let tx = fake_tx(seed, &[Some(prevout)], &[(p2wpkh_script(seed + 1), 1_234)]);
    let txid = tx.txid;
    let mut info = fake_entry_info(txid, fee, vsize);
    info.depends = parents.to_vec();
    let entry = TxEntry::new(&info, vsize, false);
    let state = mempool.test_state_mut();
    state.txs.insert(tx, entry);
    txid
}

fn publish(mempool: &mut Mempool, txids: &[Txid]) {
    mempool.test_tick(txids, FeeRate::new(1.0));
}

#[test]
fn cpfp_ancestor_and_descendant_walks_are_directional() {
    // chain: A -> B -> C
    let mut mempool = Mempool::for_test();
    let a = insert_with_depends(&mut mempool, 0xB3, 100, 100, &[]);
    let b = insert_with_depends(&mut mempool, 0xB4, 100, 100, &[a]);
    let c = insert_with_depends(&mut mempool, 0xB5, 5_800, 100, &[b]);
    publish(&mut mempool, &[a, b, c]);

    for txid in [a, b, c] {
        assert_eq!(
            mempool
                .published()
                .cpfp_info(&txid, &BlockHash::default())
                .unwrap()
                .unwrap()
                .effective_fee_per_vsize,
            FeeRate::new(20.0)
        );
    }

    // B sees A as an ancestor and C as a descendant.
    let info_b = mempool
        .published()
        .cpfp_info(&b, &BlockHash::default())
        .unwrap()
        .unwrap();
    let ancestor_ids: Vec<_> = info_b.ancestors.iter().map(|e| e.txid).collect();
    let descendant_ids: Vec<_> = info_b.descendants.iter().map(|e| e.txid).collect();
    assert_eq!(ancestor_ids, vec![a]);
    assert_eq!(descendant_ids, vec![c]);
    // best_descendant picks the highest-rate descendant.
    assert_eq!(info_b.best_descendant.as_ref().map(|e| e.txid), Some(c));
}

#[test]
fn cpfp_keeps_a_compatible_graph_during_private_changes() {
    let mut mempool = Mempool::for_test();
    let txid = insert_with_depends(&mut mempool, 0xB0, 100, 100, &[]);
    let tip = BlockHash::default();
    publish(&mut mempool, &[txid]);
    assert!(
        mempool
            .published()
            .cpfp_info(&txid, &tip)
            .unwrap()
            .is_some()
    );
    assert!(
        mempool
            .published()
            .cpfp_info(&txid, &"11".repeat(32).parse().unwrap())
            .is_err()
    );
    let child = insert_with_depends(&mut mempool, 0xB1, 100, 100, &[txid]);
    // Private membership changes cannot race the graph held by readers.
    assert!(
        mempool
            .published()
            .cpfp_info(&txid, &tip)
            .unwrap()
            .unwrap()
            .descendants
            .is_empty()
    );
    publish(&mut mempool, &[txid, child]);
    assert_eq!(
        mempool
            .published()
            .cpfp_info(&txid, &tip)
            .unwrap()
            .unwrap()
            .descendants
            .len(),
        1
    );
}
