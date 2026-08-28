use bitview::ImportContext;
use bitview_default::DefaultPlugins;
use bitview_plugin::PluginId;
use bitview_query::Vecs;
use bitview_types::SeriesName;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Index;

#[test]
fn derives_mutable_series_gates_from_vec_type() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(assert_mutable_series_gates_from_vec_type)
        .unwrap()
        .join()
        .unwrap();
}

fn assert_mutable_series_gates_from_vec_type() {
    let directory = tempfile::tempdir().unwrap();
    let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
    let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
    let context = ImportContext::new(directory.path());
    let plugins = DefaultPlugins::import(context, &reader).unwrap();
    let vecs = Vecs::build(&plugins);

    let txin_index = SeriesName::from("txin_index");
    let spent = vecs.entry(&txin_index, Index::TxOutIndex).unwrap();
    let outputs = spent.plugin();
    assert_eq!(outputs.id(), PluginId::new("outputs"));
    assert!(spent.vec().is_mutable());
    assert!(spent.requires_gate());

    let identity = vecs.entry(&txin_index, Index::TxInIndex).unwrap();
    let mappings = identity.plugin();
    assert_eq!(mappings.id(), PluginId::new("mappings"));
    assert!(!identity.vec().is_mutable());
    assert!(!identity.requires_gate());

    let reported_metric = vecs
        .entry(
            &SeriesName::from("utxos_over_5m_old_transfer_volume_average_1y_cents"),
            Index::Day1,
        )
        .unwrap();
    assert!(!reported_metric.vec().is_mutable());
    assert!(!reported_metric.requires_gate());

    let addr_state = SeriesName::from("addr_state");
    let address = vecs.entry(&addr_state, Index::P2AAddrIndex).unwrap();
    let distribution = address.plugin();
    assert_eq!(distribution.id(), PluginId::new("distribution"));
    assert!(address.vec().is_mutable());
    assert!(address.requires_gate());
}
