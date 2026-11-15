use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub blocks: &'a BlocksVecs,
    pub transactions: &'a TransactionsVecs,
    pub mining: &'a MiningVecs,
    pub mappings: &'a MappingsVecs,
}
