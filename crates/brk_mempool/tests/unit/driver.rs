use std::{
    net::TcpListener,
    panic::catch_unwind,
    sync::{Barrier, mpsc},
};

use brk_rpc::{Auth, Client};
use brk_types::{AddrBytes, FeeRate, Sats, Vin};

use super::*;
use crate::{
    state::TxEntry,
    test_support::{fake_entry_info, fake_tx, p2wpkh_script},
};

#[test]
fn complete_membership_serves_outputs_while_inputs_remain_unresolved() {
    let mut mempool = Mempool::for_test();
    let tip = BlockHash::default();
    assert!(matches!(
        mempool.published().info(),
        Err(Error::StateUpdating)
    ));
    mempool.test_publish(tip);
    let reader = mempool.read_only_clone();
    let empty = reader.load();
    assert_eq!(empty.info().unwrap().count, 0);

    let tx = fake_tx(1, &[None], &[(p2wpkh_script(2), 1234)]);
    let txid = tx.txid;
    let state = &mut mempool.state;
    state.info.add(&tx, 100_u64.into());
    state.txs.insert(
        tx,
        TxEntry::new(&fake_entry_info(txid, 100, 100), 100, false),
    );
    let (send, receive) = mpsc::channel();
    let reading = thread::spawn(move || send.send(reader.load().info().unwrap().count).unwrap());
    assert_eq!(receive.recv_timeout(Duration::from_secs(1)).unwrap(), 0);
    reading.join().unwrap();
    mempool
        .rebuilder
        .tick(&mempool.state, &[txid], FeeRate::new(1.0));
    mempool.publish_observation(tip, false);
    assert_eq!(
        mempool.published().info().unwrap().count,
        0,
        "incomplete membership is retained"
    );
    assert_eq!(
        mempool
            .published()
            .block_template_source()
            .build()
            .unwrap()
            .transactions
            .len(),
        1,
        "exact GBT advances independently"
    );
    mempool.publish_observation(tip, true);
    let first = mempool.published();
    assert_eq!(first.info().unwrap().count, 1);
    assert_eq!(
        first.live_raw_histogram(&tip).unwrap().iter().sum::<u32>(),
        1
    );
    assert_eq!(
        first
            .live_eligible_histogram(&tip)
            .unwrap()
            .iter()
            .sum::<u32>(),
        1
    );
    assert_eq!(first.txids_with_hash().unwrap().0, vec![txid]);
    let addr = AddrBytes::try_from(&p2wpkh_script(2)).unwrap();
    assert!(matches!(
        first.addr_stats(&addr, &tip),
        Err(Error::StateUpdating)
    ));
    assert!(matches!(
        first.transaction(&txid, &tip),
        Err(Error::StateUpdating)
    ));
    assert_eq!(empty.info().unwrap().count, 0);

    let record = mempool.state.txs.remove_by_prefix(&txid.into()).unwrap();
    mempool.state.info.remove(&record.tx, record.entry.fee);
    assert_eq!(mempool.published().info().unwrap().count, 1);
    mempool.test_publish(tip);
    assert_eq!(mempool.published().info().unwrap().count, 0);
    assert_eq!(first.info().unwrap().count, 1);
}

#[test]
fn fills_isolate_published_bodies_and_reuse_unchanged_bodies() {
    let mut mempool = Mempool::for_test();
    let tip = BlockHash::default();
    let unresolved = fake_tx(1, &[None], &[]);
    let unchanged = fake_tx(2, &[], &[]);
    let txid = unresolved.txid;
    let other = unchanged.txid;
    for tx in [unresolved, unchanged] {
        let entry = TxEntry::new(&fake_entry_info(tx.txid, 100, 100), 100, false);
        mempool.state.txs.insert(tx, entry);
    }
    mempool.test_publish(tip);
    let before = mempool.published();
    let old = &before.pool().unwrap().txs.record(&txid).unwrap().tx;
    let stable = &before.pool().unwrap().txs.record(&other).unwrap().tx;
    mempool.state.txs.apply_fills(
        &txid.into(),
        vec![(
            Vin::from(0usize),
            TxOut::from((p2wpkh_script(3), Sats::from(1234u64))),
        )],
    );
    assert!(old.input[0].prevout.is_none());
    mempool.test_publish(tip);
    let after = mempool.published();
    let changed = after.transaction(&txid, &tip).unwrap().unwrap();
    assert!(!Arc::ptr_eq(old, &changed));
    assert!(changed.input[0].prevout.is_some());
    assert!(Arc::ptr_eq(
        stable,
        &after.transaction(&other, &tip).unwrap().unwrap()
    ));
    assert_eq!(
        after.pool().unwrap().graph.content_revision(),
        after.pool().unwrap().txs.content_revision()
    );
}

#[test]
fn unchanged_publications_reuse_root_and_unheld_versions_are_released() {
    let mut mempool = Mempool::for_test();
    let tip = BlockHash::default();
    mempool.test_publish(tip);
    let before = mempool.published();
    let weak = Arc::downgrade(&before);
    mempool.test_publish(tip);
    assert!(Arc::ptr_eq(&before, &mempool.published()));
    mempool.test_publish("11".repeat(32).parse().unwrap());
    assert!(weak.upgrade().is_some());
    drop(before);
    assert!(weak.upgrade().is_none());
}

#[test]
fn recovery_discards_partial_private_changes_without_replacing_the_publication() {
    let mut mempool = Mempool::for_test();
    mempool.test_publish(BlockHash::default());
    let before = mempool.published();
    let result = catch_unwind(AssertUnwindSafe(|| {
        mempool.needs_recovery = true;
        let tx = fake_tx(1, &[None], &[]);
        mempool.state.info.add(&tx, 100_u64.into());
        panic!("partial update");
    }));
    assert!(result.is_err());
    mempool.restore_published();
    assert_eq!(mempool.state.info.count, 0);
    assert_eq!(mempool.state.txs.len(), 0);
    assert!(Arc::ptr_eq(&before, &mempool.published()));
    assert!(!mempool.needs_recovery);
    assert_eq!(mempool.stats().rebuilds, before.stats().rebuilds);
}

#[test]
fn fetch_failure_preserves_the_previous_publication() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let mut mempool = Mempool::for_test();
    mempool.client =
        Client::new_with(&format!("http://{address}"), Auth::None, 0, Duration::ZERO).unwrap();
    mempool.test_publish(BlockHash::default());
    let before = mempool.published();
    assert!(mempool.tick_with(|_| FxHashMap::default()).is_err());
    assert!(Arc::ptr_eq(&before, &mempool.published()));
}

#[test]
fn projection_only_changes_share_membership_containers() {
    let mut writer = Mempool::for_test();
    let tx = fake_tx(1, &[], &[]);
    let txid = tx.txid;
    writer.state.txs.insert(
        tx,
        TxEntry::new(&fake_entry_info(txid, 100, 100), 100, false),
    );
    writer.test_tick(&[txid], FeeRate::new(1.0));
    let before = writer.published();
    writer.test_tick(&[txid], FeeRate::new(2.0));
    let after = writer.published();
    let old = before.pool().unwrap();
    let new = after.pool().unwrap();
    assert!(!Arc::ptr_eq(&old.graph, &new.graph));
    assert!(Arc::ptr_eq(&old.txs, &new.txs));
    assert!(Arc::ptr_eq(&old.addrs, &new.addrs));
    assert!(Arc::ptr_eq(&old.outpoint_spends, &new.outpoint_spends));
    assert!(Arc::ptr_eq(&old.graveyard, &new.graveyard));
}

#[test]
fn concurrent_readers_observe_one_complete_membership_version() {
    let mut writer = Mempool::for_test();
    let tip = BlockHash::default();
    writer.test_publish(tip);
    let reader = writer.read_only_clone();
    let ready = Arc::new(Barrier::new(2));
    let start = ready.clone();
    let reading = thread::spawn(move || {
        start.wait();
        for _ in 0..1000 {
            let view = reader.load();
            let count = view.info().unwrap().count;
            assert_eq!(view.txids_with_hash().unwrap().0.len(), count);
            assert_eq!(
                view.live_raw_histogram(&tip).unwrap().iter().sum::<u32>() as usize,
                count
            );
            assert_eq!(
                view.pool().unwrap().graph.content_revision(),
                view.pool().unwrap().txs.content_revision()
            );
        }
    });
    ready.wait();
    for seed in 1..=100 {
        let tx = fake_tx(seed, &[], &[(p2wpkh_script(seed), 1234)]);
        writer.state.info.add(&tx, 100_u64.into());
        let entry = TxEntry::new(&fake_entry_info(tx.txid, 100, 100), 100, false);
        writer.state.txs.insert(tx, entry);
        writer.test_publish(tip);
    }
    reading.join().unwrap();
}
