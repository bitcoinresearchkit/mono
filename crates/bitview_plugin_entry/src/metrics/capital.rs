use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::metrics::CapitalViews;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyFiatPerBlock, LazyWindowStartVec};
use brk_types::{Cents, Height, Version};
use vecdb::ReadableBoxedVec;

use super::Sources;

#[derive(Clone, Traversable)]
pub struct CapitalMetrics {
    #[traversable(flatten)]
    views: CapitalViews,
    /// Capital whose creation price is at or below current spot price.
    in_profit: LazyFiatPerBlock<Cents>,
    /// Capital whose creation price is above current spot price.
    in_loss: LazyFiatPerBlock<Cents>,
}

impl CapitalMetrics {
    pub(super) fn new(
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        all_capital: &ReadableBoxedVec<Height, Cents>,
    ) -> Self {
        let name = |metric: &str| CohortContext::Utxo.metric_name(id, metric);
        let fiat = |metric: &str, source| {
            LazyFiatPerBlock::from_cents_source(&name(metric), version, source, mappings)
        };
        Self {
            views: CapitalViews::new(
                name,
                version,
                &sources.realized_cap,
                all_capital,
                mappings,
                windows,
            ),
            in_profit: fiat("capital_in_profit", &sources.capital_in_profit),
            in_loss: fiat("capital_in_loss", &sources.capital_in_loss),
        }
    }
}
