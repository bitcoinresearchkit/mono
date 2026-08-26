use bitview_plugin_indexer::Indexer;
use bitview_plugin_transactions::FeesVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub fees: &'a FeesVecs,
}
