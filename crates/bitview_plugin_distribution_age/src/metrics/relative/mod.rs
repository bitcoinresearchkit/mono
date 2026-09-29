use bitview_cohort::{CohortContext, UTXOAggregate};
use bitview_compute::FixedRatio;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{CachedSeries, LazyPercentPerBlock};
use brk_types::{Height, PartsPerMillion32, Version};
use vecdb::LazyVec;

mod gross_pnl_composition;
mod source;
mod supply_profitability_shares;
mod vecs;

pub use gross_pnl_composition::GrossPnlComposition;
pub use source::RelativeSource;
pub use supply_profitability_shares::SupplyProfitabilityShares;
pub use vecs::RelativeVecs;

fn public_profit_share(_: Height, profit_share: PartsPerMillion32) -> PartsPerMillion32 {
    if profit_share.is_nan() {
        PartsPerMillion32::ZERO
    } else {
        profit_share
    }
}

fn public_loss_share(_: Height, profit_share: PartsPerMillion32) -> PartsPerMillion32 {
    if profit_share.is_nan() {
        PartsPerMillion32::ZERO
    } else {
        PartsPerMillion32::ONE - profit_share
    }
}

fn share_views<B: FixedRatio>(
    sources: &UTXOAggregate<CachedSeries<Height, PartsPerMillion32>>,
    metric: &str,
    version: Version,
    compute: fn(Height, PartsPerMillion32) -> B,
    mappings: &MappingsVecs,
) -> UTXOAggregate<LazyPercentPerBlock<B>> {
    UTXOAggregate::from_fn(|id| {
        let name = CohortContext::Utxo.metric_name(id.cohort(), metric);
        let source = id.select(sources);
        let source = LazyVec::init(
            &format!("{name}_{}_source", B::SUFFIX),
            version,
            source.read_only_boxed_clone(),
            compute,
        );
        LazyPercentPerBlock::from_height_source(&name, version, &source, mappings)
    })
}
use vecdb::ReadableCloneableVec;
