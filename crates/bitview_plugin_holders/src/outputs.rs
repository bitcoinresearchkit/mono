use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{Count, CountSigned, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_types::Version;

use crate::columns::Columns;

#[derive(Clone, Traversable)]
pub struct Outputs {
    /// Number of transaction outputs that are unspent at the represented block.
    pub unspent_count: LazyPerBlockWithDeltas<Count, CountSigned, PartsPerMillionSigned64>,
    /// Number of the cohort's outputs spent in each block.
    pub spent_count: LazyPerBlockCumulativeRolling<Count>,
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
