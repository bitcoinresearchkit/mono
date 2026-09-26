use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    thread,
    time::Duration,
};

use bitview_plugin_indexer::HasIndexer;
use bitview_plugin_transactions::HasTransactions;
use brk_types::TxIndex;
use serde_json::{Value, from_str};
use tokio::task::{self, JoinSet};
use vecdb::{AnyVec, ReadableCloneableVec, ReadableVec, VecValue};

use super::{
    chain_fixture::{default_first, run_genesis},
    server_routes::exchange_with_etag,
};
use crate::test_cache::init_cache;

fn assert_uncached<T: VecValue + PartialEq>(source: &impl ReadableCloneableVec<TxIndex, T>) {
    assert!(!source.is_empty());
    let reader = source.read_only_boxed_clone();
    assert_eq!(source.collect(), reader.collect());
    assert!(!source.read_cached_into_at(0, source.len(), &mut Vec::new()));
    assert!(!reader.read_cached_into_at(0, reader.len(), &mut Vec::new()));
}

#[test]
fn production_transaction_flags_and_fee_sources_remain_uncached() {
    init_cache();
    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(1, 1);
        let tx = fixture.plugins.transactions();
        assert_uncached(&tx.patterns.flags.is_coinjoin);
        assert_uncached(&tx.patterns.flags.is_consolidation);
        assert_uncached(&tx.patterns.flags.is_batch_payout);
        assert_uncached(&tx.fees.cpfp_flags.is_cpfp_parent);
        assert_uncached(&tx.fees.cpfp_flags.is_cpfp_child);
        assert_uncached(&tx.fees.fee.tx_index);
        assert_uncached(&tx.fees.fee_rate);
        assert_uncached(&tx.fees.effective_fee_rate.tx_index);
        // Ensure the process-wide budget is active for a real cached source.
        let timestamp = &fixture.plugins.indexer().vecs().blocks.timestamp;
        timestamp.collect();
        assert!(timestamp.read_cached_into_at(0, timestamp.len(), &mut Vec::new()));
    });
}

async fn chart(address: SocketAddr) -> Vec<Value> {
    let response = exchange_with_etag(
        address,
        "GET",
        "/api/series/timestamp/hour4/data?from=-2",
        "\"old\"",
    )
    .await;
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[test]
fn chart_reads_survive_concurrent_cache_eviction_and_reorgs() {
    init_cache();

    run_genesis(default_first(), |mut fixture| async move {
        fixture.publish(1, 1);
        let address = fixture.address;
        let first = chart(address).await;
        fixture.publish(2, 1);
        let second = chart(address).await;
        assert_ne!(first, second);
        fixture.publish(1, 1);
        let allowed = Arc::new([first, second]);
        let stop = Arc::new(AtomicBool::new(false));
        let evict_stop = stop.clone();
        let evictor = thread::spawn(move || {
            while !evict_stop.load(Relaxed) {
                init_cache().clear();
                thread::sleep(Duration::from_millis(1));
            }
        });
        let mut readers = JoinSet::new();
        for _ in 0..4 {
            let allowed = allowed.clone();
            readers.spawn(async move {
                for _ in 0..16 {
                    let values = chart(address).await;
                    assert!(allowed.contains(&values), "mixed publication: {values:?}");
                }
            });
        }
        // Run real fixture branch replacement concurrently with the HTTP tasks.
        let writer = task::spawn_blocking(move || {
            for branch in [2, 1, 2, 1] {
                fixture.publish(branch, 1);
            }
            fixture
        });
        let mut errors = Vec::new();
        while let Some(result) = readers.join_next().await {
            if let Err(error) = result {
                errors.push(error);
            }
        }
        let fixture = writer.await;
        stop.store(true, Relaxed);
        evictor.join().unwrap();
        let fixture = fixture.unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(
            fixture.query.sync(|q| q.tip_blockhash()).to_string(),
            fixture.chain[1].block_hash().to_string()
        );
        assert_eq!(chart(address).await, allowed[0]);
    });
}
