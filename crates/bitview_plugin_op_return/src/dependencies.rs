use bitview_plugin_indexer::Indexer;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub fees: &'a bitview_plugin_transactions::FeesVecs,
}
