use std::path::Path;

use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use vecdb::Budgeted;

pub fn import_indexer(data_dir: &Path) -> Indexer {
    Budgeted::init_global(bitview_plugin::DEFAULT_CACHE_BUDGET).unwrap();
    let node = ConnectArgs::default();
    let client = node.client().expect("Failed to connect to Bitcoin Core");
    let reader = Reader::new(node.blocks_dir(), &client);
    let context = ImportContext::new(data_dir);
    Indexer::import(context, &reader).expect("Failed to import indexer")
}
