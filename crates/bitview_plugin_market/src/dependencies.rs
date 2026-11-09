use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;

pub struct Dependencies<'a> {
    pub price: &'a PriceVecs,
    pub mappings: &'a MappingsVecs,
}
