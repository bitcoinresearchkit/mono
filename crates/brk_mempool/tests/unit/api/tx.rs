use std::sync::Arc;

use brk_types::{BlockHash, FeeRate, TxOutspend, Vout};

use crate::{
    Mempool, TxRemoval,
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, fake_txid, p2wpkh_script},
};

#[test]
fn bulk_outspends_resolve_fan_in_and_preserve_existing_spends() {
    let mut mempool = Mempool::for_test();
    let count = 1000;
    let parent = fake_tx(1, &[], &vec![(p2wpkh_script(1), 1_000); count]);
    let mut spender = fake_tx(2, &[None], &[(p2wpkh_script(2), 900)]);
    spender.input.resize(count, spender.input[0].clone());
    for (vin, input) in spender.input.iter_mut().enumerate() {
        input.txid = parent.txid;
        input.vout = Vout::from(count - vin - 1);
        input.prevout = Some(parent.output[count - vin - 1].clone());
    }
    let tip = BlockHash::default();
    {
        let state = mempool.test_state_mut();
        let entry = TxEntry::new(&fake_entry_info(spender.txid, 100, 100), 100, false);
        state
            .outpoint_spends
            .insert_spends(&spender, entry.txid_prefix());
        state.txs.insert(spender.clone(), entry);
        state.txs.insert(
            parent.clone(),
            TxEntry::new(&fake_entry_info(parent.txid, 100, 100), 100, false),
        );
    }
    mempool.test_publish(tip);
    let outspends = mempool
        .published()
        .outspends_if_present(&parent.txid, &tip)
        .unwrap()
        .unwrap();
    assert_eq!(outspends.len(), count);
    for (vout, outspend) in outspends.iter().enumerate() {
        assert_eq!(outspend.txid, Some(spender.txid));
        assert_eq!(outspend.vin, Some((count - vout - 1).into()));
    }
    let mut overlay = vec![TxOutspend::UNSPENT; count];
    overlay[0] = outspends[0].clone();
    overlay[0].txid = Some(fake_txid(3));
    mempool
        .published()
        .merge_outspends(&parent.txid, &mut overlay, &tip)
        .unwrap();
    assert_eq!(overlay[0].txid, Some(fake_txid(3)));
    for outspend in &overlay[1..] {
        assert_eq!(outspend.txid, Some(spender.txid));
    }
}

#[test]
fn transaction_reads_reject_unready_or_wrong_chain_and_share_vanished_bodies() {
    let mut mempool = Mempool::for_test();
    let tip = BlockHash::default();
    let wrong_tip = "11".repeat(32).parse().unwrap();
    let tx = fake_tx(1, &[], &[(p2wpkh_script(1), 1_234)]);
    let txid = tx.txid;
    let reader = mempool.read_only_clone();
    let assert_unready = |tip| {
        assert!(reader.load().transaction(&txid, tip).is_err());
        assert!(reader.load().contains_txid(&txid, tip).is_err());
        assert!(
            reader
                .load()
                .outspend_if_present(&txid, Vout::ZERO, tip)
                .is_err()
        );
        assert!(reader.load().outspends_if_present(&txid, tip).is_err());
        assert!(reader.load().outspend(&txid, Vout::ZERO, tip).is_err());
        assert!(reader.load().outspend(&txid, Vout::ZERO, tip).is_err());
        assert!(
            reader
                .load()
                .merge_outspends(&txid, &mut [TxOutspend::UNSPENT], tip)
                .is_err()
        );
    };
    assert_unready(&tip);
    {
        let state = mempool.test_state_mut();
        state.txs.insert(
            tx,
            TxEntry::new(&fake_entry_info(txid, 100, 100), 100, false),
        );
    }
    mempool.test_publish(tip);
    assert_unready(&wrong_tip);
    let body = mempool
        .published()
        .transaction(&txid, &tip)
        .unwrap()
        .unwrap();
    assert!(mempool.published().contains_txid(&txid, &tip).unwrap());
    {
        let state = mempool.test_state_mut();
        let record = state.txs.remove_by_prefix(&txid.into()).unwrap();
        state.graveyard.bury(
            record.tx,
            record.entry,
            FeeRate::new(1.0),
            TxRemoval::Vanished,
        );
    }
    mempool.test_publish(tip);
    assert!(!mempool.published().contains_txid(&txid, &tip).unwrap());
    let vanished = mempool
        .published()
        .transaction(&txid, &tip)
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&body, &vanished));
    assert!(
        mempool
            .published()
            .outspends_if_present(&txid, &tip)
            .unwrap()
            .is_none()
    );
    assert_eq!(body.output.len(), 1);
}
