use bitview_plugin_distribution::Vecs as DistributionVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_urpd::AgeRangeUrpds;

pub struct Dependencies<'a> {
    pub age_urpds: &'a AgeRangeUrpds,
    pub price: &'a PriceVecs,
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub distribution: &'a DistributionVecs,
}
