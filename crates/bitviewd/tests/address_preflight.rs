use std::{mem::discriminant, thread};

use bitview::{ImportContext, PluginSet};
use bitview_default::DefaultPlugins;
use bitview_plugin::PluginId;
use bitview_query::{AddrStatsPreflight, Query};
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Addr;

#[test]
fn address_preflight_matches_stats_resolution_and_defers_during_updates() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(assert_address_preflight_matches_stats_resolution_and_defers_during_updates)
        .unwrap()
        .join()
        .unwrap();
}

fn assert_address_preflight_matches_stats_resolution_and_defers_during_updates() {
    let directory = tempfile::tempdir().unwrap();
    let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
    let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
    let plugins = DefaultPlugins::import(ImportContext::new(directory.path()), &reader).unwrap();

    let mut distribution_gate = None;
    plugins.for_each_plugin(&mut |plugin| {
        if plugin.id() == PluginId::new("distribution") {
            distribution_gate = Some(plugin.gate().clone());
        }
    });
    let distribution_gate = distribution_gate.unwrap();
    let query = Query::build(&plugins, None);

    for raw in ["17jGLFhcnPYqG17qN2ouxbScrcnroHqRP", "not-an-address"] {
        let addr = Addr::from(raw.to_owned());
        let expected = query.addr(addr.clone()).unwrap_err();
        let AddrStatsPreflight::Reject(actual) = query.addr_stats_preflight(&addr) else {
            panic!("unknown or invalid address should be rejected");
        };
        assert_eq!(discriminant(&actual), discriminant(&expected));
    }

    distribution_gate.begin_update();
    let addr = Addr::from("17jGLFhcnPYqG17qN2ouxbScrcnroHqRP".to_owned());
    assert!(matches!(
        query.addr_stats_preflight(&addr),
        AddrStatsPreflight::Updating
    ));
    assert!(matches!(
        query.addr_stats_preflight(&Addr::from("not-an-address".to_owned())),
        AddrStatsPreflight::Reject(brk_error::Error::InvalidAddr)
    ));
    assert!(matches!(
        query.addr(Addr::from("not-an-address".to_owned())),
        Err(brk_error::Error::InvalidAddr)
    ));

    distribution_gate.finish_update();
    assert!(matches!(
        query.addr_stats_preflight(&addr),
        AddrStatsPreflight::Reject(brk_error::Error::UnknownAddr)
    ));
}
