use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_distribution::Vecs as DistributionVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_urpd::AgeRangeUrpds;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub distribution: &'a DistributionVecs,
    pub age_urpds: &'a AgeRangeUrpds,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
}
