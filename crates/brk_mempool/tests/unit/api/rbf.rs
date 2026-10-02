use brk_types::{FeeRate, TxOut, TxidPrefix};

use super::*;
use crate::{
    Mempool, TxRemoval,
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
};

/// Place a live tx (the replacer) and bury one or more predecessors
/// pointing at it. `bury_chain` carries `(seed, predecessor_of_next)`
/// pairs in oldest-first order. Each links forward to the next entry
/// or to `live_seed` when last.
fn build_rbf_world(live_seed: u8, predecessors: &[u8]) -> (Mempool, Txid, Vec<Txid>) {
    let mut mempool = Mempool::for_test();
    let live_tx = fake_tx(
        live_seed,
        &[Some(TxOut::from((p2wpkh_script(99), Sats::from(6_234u64))))],
        &[(p2wpkh_script(live_seed + 1), 1_234)],
    );
    let live_txid = live_tx.txid;
    let live_entry = TxEntry::new(&fake_entry_info(live_txid, 5_000, 100), 100, true);

    let mut pred_txids = Vec::with_capacity(predecessors.len());
    let state = mempool.test_state_mut();
    for (i, seed) in predecessors.iter().enumerate() {
        let tx = fake_tx(*seed, &[None], &[(p2wpkh_script(seed + 1), 1_234)]);
        let txid = tx.txid;
        // Each predecessor signals BIP-125 (rbf=true) so full_rbf stays clear.
        let entry = TxEntry::new(&fake_entry_info(txid, 1_000, 100), 100, true);
        let by = predecessors
            .get(i + 1)
            .map(|next_seed| fake_txid(*next_seed))
            .unwrap_or(live_txid);
        let rate = FeeRate::from((entry.fee, entry.vsize));
        state
            .graveyard
            .bury(tx, entry, rate, TxRemoval::Replaced { by });
        pred_txids.push(txid);
    }
    state.txs.insert(live_tx, live_entry);
    mempool.test_tick(&[live_txid], FeeRate::new(1.0));
    assert!(
        mempool
            .published()
            .pool()
            .unwrap()
            .ensure_resolved_at(&BlockHash::default())
            .is_ok()
    );
    (mempool, live_txid, pred_txids)
}

#[test]
fn rbf_requires_matching_publication_and_graph_revision() {
    let tip = BlockHash::default();
    let empty = Mempool::for_test();
    assert!(matches!(
        empty.published().rbf_for_tx(&Txid::COINBASE, &tip),
        Err(Error::StateUpdating)
    ));
    for limit in [0, 25] {
        assert!(matches!(
            empty.published().recent_rbf_trees(false, limit, &tip),
            Err(Error::StateUpdating)
        ));
    }

    let (mut mempool, live, predecessors) = build_rbf_world(200, &[1]);
    let root = mempool
        .published()
        .rbf_for_tx(&live, &tip)
        .unwrap()
        .root
        .unwrap();
    assert_eq!(
        root.rate,
        mempool
            .published()
            .snapshot()
            .chunk_rate_for(&live)
            .unwrap()
    );
    let wrong_tip = "11".repeat(32).parse().unwrap();
    assert!(matches!(
        mempool.published().rbf_for_tx(&live, &wrong_tip),
        Err(Error::StateUpdating)
    ));
    assert!(matches!(
        mempool.published().recent_rbf_trees(false, 25, &wrong_tip),
        Err(Error::StateUpdating)
    ));

    // A completed live revision must not borrow rates from the older graph.
    let state = mempool.test_state_mut();
    state.txs.remove_by_prefix(&TxidPrefix::from(live)).unwrap();

    assert_eq!(
        mempool
            .published()
            .rbf_for_tx(&predecessors[0], &tip)
            .unwrap()
            .root
            .unwrap()
            .txid,
        live
    );
    assert_eq!(
        mempool
            .published()
            .recent_rbf_trees(false, 25, &tip)
            .unwrap()
            .len(),
        1
    );
    mempool.test_tick(&[], FeeRate::new(1.0));
    assert!(
        mempool
            .published()
            .rbf_for_tx(&predecessors[0], &tip)
            .unwrap()
            .is_empty()
    );
    assert!(
        mempool
            .published()
            .recent_rbf_trees(false, 25, &tip)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rbf_for_tx_chain_walks_to_terminal_root() {
    for predecessors in [&[0xC3][..], &[0xC3, 0xC4][..]] {
        let (mempool, live, preds) = build_rbf_world(0xC2, predecessors);
        let rbf = mempool
            .published()
            .rbf_for_tx(&preds[0], &BlockHash::default())
            .unwrap();
        let root = rbf.root.expect("terminal replacer reachable");
        assert_eq!(root.txid, live);
        assert!(root.in_mempool);
        let mut node = &root;
        for txid in preds.iter().rev() {
            assert_eq!(node.replaces.len(), 1);
            node = &node.replaces[0];
            assert_eq!(node.txid, *txid);
            assert!(!node.in_mempool);
        }
        assert!(node.replaces.is_empty());
        assert!(rbf.replaces.is_empty());
    }
}

#[test]
fn recent_rbf_trees_dedup_by_root_and_respect_limit() {
    // Chain 0xC6 -> 0xC7 -> live plus a sibling 0xC8 also replaced by
    // live. All paths roll up to the same root, so the recent listing
    // dedups them down to a single tree.
    let (mut mempool, live, _preds) = build_rbf_world(0xC5, &[0xC6, 0xC7]);
    {
        let state = mempool.test_state_mut();
        let extra = fake_tx(0xC8, &[None], &[(p2wpkh_script(0xC9), 1_234)]);
        let extra_txid = extra.txid;
        let entry = TxEntry::new(&fake_entry_info(extra_txid, 999, 100), 100, true);
        let rate = FeeRate::from((entry.fee, entry.vsize));
        state
            .graveyard
            .bury(extra, entry, rate, TxRemoval::Replaced { by: live });
    }
    mempool.test_publish(BlockHash::default());
    let trees = mempool
        .published()
        .recent_rbf_trees(false, 10, &BlockHash::default())
        .unwrap();
    assert_eq!(trees.len(), 1, "all paths roll up to one root");
    assert_eq!(trees[0].txid, live);

    let capped = mempool
        .published()
        .recent_rbf_trees(false, 0, &BlockHash::default())
        .unwrap();
    assert!(capped.is_empty(), "limit honored");
}
