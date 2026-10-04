use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillionSigned64, PriceRatio, Ratio};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeRolling, LazyFiatPerBlockCumulativeWithSums,
    LazyFiatPerBlockCumulativeWithSumsAndDeltas, LazyFiatPerBlockWithDeltas, LazyPerBlock,
    LazyPriceWithRatioPerBlock, LazyWindowStartVec, PerBlock,
};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{Database, Ident, ReadableBoxedVec, Rw, StorageMode};

use super::Sources;

#[derive(Traversable)]
pub struct RealizedMetrics<M: StorageMode = Rw> {
    /// Creation-date value of this cohort's unspent outputs.
    cap: LazyFiatPerBlockWithDeltas<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// Satoshi-weighted creation price of this cohort's unspent outputs.
    pub(crate) price: LazyPriceWithRatioPerBlock,
    /// Profit realized by outputs spent from this cohort.
    profit: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Loss realized by outputs spent from this cohort.
    loss: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Realized profit minus realized loss.
    net_pnl: LazyFiatPerBlockCumulativeWithSumsAndDeltas<
        CentsSigned,
        CentsSigned,
        PartsPerMillionSigned64,
    >,
    /// Spending value divided by creation-date value over the trailing 24 hours.
    pub(crate) sopr: PerBlock<Ratio, M>,
    #[traversable(wrap = "sopr")]
    /// Creation-date value of outputs spent from this cohort.
    pub(crate) value_destroyed: LazyFiatPerBlockCumulativeRolling<Cents>,
    /// Spot price divided by this cohort's realized price.
    mvrv: LazyPerBlock<Ratio>,
}

impl RealizedMetrics {
    pub(crate) fn import(
        db: &Database,
        id: CohortId,
        version: Version,
        sources: &Sources,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = |metric| CohortContext::Utxo.metric_name(id, metric);
        let price = LazyPriceWithRatioPerBlock::from_height_source(
            &name("realized_price"),
            version,
            &sources.realized_price,
            mappings,
            prices,
        );
        let loss = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            &name("realized_loss"),
            version,
            sources.realized_loss.cumulative_source(),
            mappings,
            windows,
        );
        let mvrv = LazyPerBlock::from_lazy::<Ident, PriceRatio>(
            &name("mvrv"),
            version,
            &price.relative.ratio,
        );
        Ok(Self {
            cap: LazyFiatPerBlockWithDeltas::from_cents_source(
                &name("realized_cap"),
                version,
                &sources.realized_cap,
                Version::TWO,
                mappings,
                windows,
            ),
            price,
            profit: LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name("realized_profit"),
                version,
                sources.realized_profit.cumulative_source(),
                mappings,
                windows,
            ),
            loss,
            net_pnl: LazyFiatPerBlockCumulativeWithSumsAndDeltas::from_cumulative_cents_source(
                &name("net_realized_pnl"),
                version,
                sources.realized_net_pnl.cumulative_source(),
                Version::ONE,
                mappings,
                windows,
            ),
            sopr: PerBlock::import(db, &name("sopr_24h"), version, mappings)?,
            value_destroyed: LazyFiatPerBlockCumulativeRolling::from_cumulative_cents_source(
                &name("value_destroyed"),
                version,
                sources.value_destroyed.cumulative_source(),
                mappings,
                windows,
            ),
            mvrv,
        })
    }
}
