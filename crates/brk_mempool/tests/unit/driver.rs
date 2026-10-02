use std::sync::{Barrier, mpsc};

use brk_types::{AddrBytes, FeeRate};

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
