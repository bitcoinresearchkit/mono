use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;

pub struct Dependencies<'a> {
    pub mappings: &'a MappingsVecs,
    pub blocks: &'a BlocksVecs,
    pub transactions: &'a TransactionsVecs,
    pub price: &'a PriceVecs,
}
