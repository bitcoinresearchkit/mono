use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub price: &'a PriceVecs,
    pub mappings: &'a MappingsVecs,
}
