use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_urpd::StoredReplayInputs;

#[derive(Clone, Copy)]
pub struct Dependencies<'a> {
    pub urpd: StoredReplayInputs<'a>,
    pub mappings: &'a MappingsVecs,
    pub price: &'a PriceVecs,
    pub age: &'a AgeVecs,
}
