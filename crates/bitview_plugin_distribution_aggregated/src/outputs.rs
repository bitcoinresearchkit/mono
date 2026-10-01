use crate::columns::Columns;
use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};

#[derive(Clone, Traversable)]
pub struct Outputs {
    pub unspent_count: LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
    pub spent_count: LazyPerBlockCumulativeRolling<StoredU64>,
}
impl Outputs {
    pub(crate) fn new(
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let unspent_count = LazyPerBlockWithDeltas::from_height_source(
            &id.metric_name("utxo_count"),
            v,
            &c.count,
            v,
            mappings,
            windows,
        );
        let spent_count = LazyPerBlockCumulativeRolling::from_cumulative_source(
            &id.metric_name("spent_utxo_count"),
            v,
            &c.spent_count,
            windows,
            mappings,
        );
        Self {
            unspent_count,
            spent_count,
        }
    }
}
