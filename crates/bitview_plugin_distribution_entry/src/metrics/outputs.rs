use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};

use super::Sources;

#[derive(Clone, Traversable)]
pub struct OutputMetrics {
    /// Number of unspent outputs in this cohort.
    unspent_count: LazyPerBlockWithDeltas<StoredU64, StoredI64, PartsPerMillionSigned64>,
    /// Number of outputs spent from this cohort.
    pub(crate) spent_count: LazyPerBlockCumulativeRolling<StoredU64>,
}

impl OutputMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        Self {
            unspent_count: LazyPerBlockWithDeltas::from_height_source(
                &CohortContext::Utxo.metric_name(id, "utxo_count"),
                version,
                &sources.unspent_count,
                Version::TWO,
                mappings,
                windows,
            ),
            spent_count: LazyPerBlockCumulativeRolling::from_cumulative_source(
                &CohortContext::Utxo.metric_name(id, "spent_utxo_count"),
                version,
                sources.spent_count.cumulative_source(),
                windows,
                mappings,
            ),
        }
    }
}
