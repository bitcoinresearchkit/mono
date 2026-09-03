use std::{
    mem::discriminant,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use bitview::{ImportContext, PluginSet};
use bitview_default::DefaultPlugins;
use bitview_plugin::PluginId;
use bitview_query::{AddrMempoolTxsPreflight, Query};
use brk_error::Error;
use brk_mempool::Mempool;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::{Addr, BlockHash, Day1, NextBlockHash, Txid};

#[test]
fn query_preflights_preserve_resolution_errors_and_defer_during_updates() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(assert_query_preflights_preserve_resolution_errors_and_defer_during_updates)
        .unwrap()
        .join()
        .unwrap();
}

fn assert_query_preflights_preserve_resolution_errors_and_defer_during_updates() {
    let directory = tempfile::tempdir().unwrap();
    let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
    let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
    let plugins = DefaultPlugins::import(ImportContext::new(directory.path()), &reader).unwrap();

    let mut distribution_gate = None;
    let mut indexer_gate = None;
    let mut mappings_gate = None;
    plugins.for_each_plugin(&mut |plugin| {
        if plugin.id() == PluginId::new("distribution") {
            distribution_gate = Some(plugin.gate().clone());
        }
        if plugin.id() == PluginId::new("indexer") {
            indexer_gate = Some(plugin.gate().clone());
        }
        if plugin.id() == PluginId::new("mappings") {
            mappings_gate = Some(plugin.gate().clone());
        }
    });
    let distribution_gate = distribution_gate.unwrap();
    let indexer_gate = indexer_gate.unwrap();
    let mappings_gate = mappings_gate.unwrap();
    let query = Query::build(&plugins, Some(Mempool::new(&client)));

    mappings_gate.begin_update();
    thread::scope(|scope| {
        let (started_tx, started_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();
        let query = &query;
        scope.spawn(move || {
            started_tx.send(()).unwrap();
            result_tx
                .send(query.day_is_deeply_confirmed(Day1::default()))
                .unwrap();
        });

        started_rx.recv().unwrap();
        assert!(result_rx.recv_timeout(Duration::from_millis(10)).is_err());
        mappings_gate.finish_update();
        assert!(
            !result_rx
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
                .unwrap()
        );
    });

    let unknown_block = BlockHash::default();
    assert!(matches!(
        query.resolve_block(&unknown_block),
        Err(Error::NotFound(_))
    ));
    assert!(matches!(
        query.height_by_hash(&unknown_block),
        Err(Error::NotFound(_))
    ));

    let unknown_txid = Txid::COINBASE;
    assert!(matches!(
        query.resolve_confirmed_tx(&unknown_txid),
        Err(Error::UnknownTxid)
    ));
    assert!(matches!(
        query.resolve_tx(&unknown_txid),
        Err(Error::UnknownTxid)
    ));
    assert!(matches!(
        query.transaction_status(&unknown_txid),
        Err(Error::UnknownTxid)
    ));
    assert!(matches!(
        query.resolve_raw_transaction(&unknown_txid),
        Err(Error::UnknownTxid)
    ));
    assert!(matches!(
        query.resolve_transaction(&unknown_txid),
        Err(Error::UnknownTxid)
    ));
    assert!(matches!(
        query.resolve_cpfp(&unknown_txid),
        Err(Error::UnknownTxid)
    ));
    let rbf = query.resolve_rbf(&unknown_txid).unwrap();
    assert!(rbf.identity().is_some());

    assert!(matches!(
        query.block_template_diff_json_preflight(NextBlockHash::new(0xDEAD_BEEF)),
        Err(Error::NotFound(_))
    ));

    for raw in ["17jGLFhcnPYqG17qN2ouxbScrcnroHqRP", "not-an-address"] {
        let addr = Addr::from(raw.to_owned());
        let expected = query.addr(addr.clone()).unwrap_err();
        let actual = query.addr_stats_preflight(&addr).unwrap_err();
        assert_eq!(discriminant(&actual), discriminant(&expected));
        assert_eq!(
            discriminant(&query.resolve_addr_chain_txs(&addr, None, 25).unwrap_err()),
            discriminant(&expected)
        );
        let actual = match query.addr_utxos_preflight(&addr) {
            Ok(_) => panic!("unknown or invalid address should be rejected"),
            Err(error) => error,
        };
        assert_eq!(discriminant(&actual), discriminant(&expected));
    }

    let unknown_addr = Addr::from("17jGLFhcnPYqG17qN2ouxbScrcnroHqRP".to_owned());
    let AddrMempoolTxsPreflight::Cached(first, _) = query
        .addr_mempool_txs_json_preflight(&unknown_addr, 50)
        .unwrap()
    else {
        panic!("address without mempool activity should use the shared empty response");
    };
    let AddrMempoolTxsPreflight::Cached(second, _) = query
        .addr_mempool_txs_json_preflight(&unknown_addr, 50)
        .unwrap()
    else {
        panic!("address without mempool activity should remain cached");
    };
    assert_eq!(&*first, b"[]");
    assert!(Arc::ptr_eq(&first, &second));

    let invalid_addr = Addr::from("not-an-address".to_owned());
    assert!(matches!(
        query.addr_mempool_txs_json_preflight(&invalid_addr, 50),
        Err(Error::InvalidAddr)
    ));
    assert!(matches!(
        query.addr_txs_json_preflight(&unknown_addr, 50, 25, 50),
        Err(Error::UnknownAddr)
    ));
    assert!(matches!(
        query.addr_txs_json_preflight(&invalid_addr, 50, 25, 50),
        Err(Error::InvalidAddr)
    ));

    indexer_gate.begin_update();
    assert!(matches!(
        query.addr_utxos_preflight(&unknown_addr),
        Ok(None)
    ));
    assert!(matches!(
        query.addr_utxos_preflight(&invalid_addr),
        Err(Error::InvalidAddr)
    ));
    indexer_gate.finish_update();

    distribution_gate.begin_update();
    let addr = Addr::from("17jGLFhcnPYqG17qN2ouxbScrcnroHqRP".to_owned());
    assert!(matches!(query.addr_stats_preflight(&addr), Ok(None)));
    assert!(matches!(
        query.addr_stats_preflight(&Addr::from("not-an-address".to_owned())),
        Err(brk_error::Error::InvalidAddr)
    ));
    assert!(matches!(
        query.addr(Addr::from("not-an-address".to_owned())),
        Err(brk_error::Error::InvalidAddr)
    ));

    distribution_gate.finish_update();
    assert!(matches!(
        query.addr_stats_preflight(&addr),
        Err(brk_error::Error::UnknownAddr)
    ));
}
