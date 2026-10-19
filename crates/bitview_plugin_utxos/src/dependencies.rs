use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use brk_types::Version;
use vecdb::AnyVec;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub mappings: &'a MappingsVecs,
    pub price: &'a PriceVecs,
}

impl Dependencies<'_> {
    pub(crate) fn version(&self) -> Version {
        let v = self.indexer.vecs();
        Version::ONE
            + Version::combine_all([
                self.price.spot.cents.height.version(),
                v.inputs.value.version(),
                v.outputs.first_txout_index.version(),
                v.inputs.first_txin_index.version(),
                v.outputs.value.version(),
                v.outputs.output_type.version(),
                v.inputs.txout_index.version(),
                v.inputs.output_type.version(),
            ])
    }
}
