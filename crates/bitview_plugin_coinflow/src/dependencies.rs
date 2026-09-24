use bitview_plugin_distribution::{UTXOStates, Vecs as DistributionVecs};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub utxo_states: &'a UTXOStates,
    pub price: &'a PriceVecs,
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub distribution: &'a DistributionVecs,
}
