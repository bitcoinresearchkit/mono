use bitview_cohort::AgeAggregateId;
use bitview_collections::ByPercentile;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{
    Density, LazyPerBlock, LazyPercentPerBlock, PerBlock, Price, PriceWithMvrv, PriceWithRatio,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, Ident, ReadableVec, Rw, StorageMode};

use super::CostBasisVecs;
use crate::columns::Columns;

/// Creation-price statistics of the cohort's unspent outputs.
#[derive(Traversable)]
pub struct CostBasis<M: StorageMode = Rw> {
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by satoshis.
    pub per_coin: PerCoin<M>,
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by each output's USD value at creation.
    pub per_dollar: PerDollar<M>,
    /// Lowest creation price (spot when the output was created) among the cohort's unspent
    /// outputs.
    pub min: Price<LazyPerBlock<Cents>>,
    /// Highest creation price (spot when the output was created) among the cohort's unspent
    /// outputs.
    pub max: Price<LazyPerBlock<Cents>>,
    /// Share of the cohort's unspent supply with a creation price near spot.
    pub supply_density: Density<LazyPercentPerBlock<PartsPerMillion32>>,
    /// Share of the cohort's invested capital (satoshis times creation price) with a creation
    /// price near spot.
    pub capital_density: Density<LazyPercentPerBlock<PartsPerMillion32>>,
}

#[derive(Traversable)]
pub struct PerCoin<M: StorageMode = Rw> {
    /// The weighted mean.
    pub avg: PriceWithMvrv<M>,
    /// Realized price: realized cap divided by supply, the weighted mean.
    pub realized_price: Price<LazyPerBlock<Cents>>,
    #[traversable(flatten)]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
    /// Median realized price: the weighted median.
    pub median_realized_price: Price<LazyPerBlock<Cents>>,
}

#[derive(Traversable)]
pub struct PerDollar<M: StorageMode = Rw> {
    /// The weighted mean.
    pub avg: PriceWithRatio<M>,
    /// Capitalized price: the weighted mean.
    pub capitalized_price: Price<LazyPerBlock<Cents>>,
    #[traversable(flatten)]
    pub percentiles: ByPercentile<Price<LazyPerBlock<Cents>>>,
}

impl CostBasis {
    /// The means are lazy over the cohort's columns, the rest over `sources`.
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        sources: &CostBasisVecs,
    ) -> Result<Self> {
        let name = |metric| id.metric_name(metric);
        let sources_v = CostBasisVecs::version(v);
        let price = |metric, source: &Price<PerBlock<Cents>>| {
            Price::from_height_source(&name(metric), sources_v, &source.cents.height, mappings)
        };
        let per_coin = id.select(&sources.per_coin);
        Ok(Self {
            per_coin: PerCoin {
                avg: PriceWithMvrv::import(
                    db,
                    &name("cost_basis_per_coin_avg"),
                    &name("mvrv"),
                    v,
                    &c.price,
                    mappings,
                )?,
                realized_price: Price::from_height_source(
                    &name("realized_price"),
                    v,
                    &c.price,
                    mappings,
                ),
                median_realized_price: Price::from_lazy_cents_source::<Ident, Cents>(
                    &name("median_realized_price"),
                    v,
                    &per_coin.median.cents,
                ),
                percentiles: per_coin.prices.clone(),
            },
            per_dollar: PerDollar {
                avg: PriceWithRatio::import(
                    db,
                    &name("cost_basis_per_dollar_avg"),
                    v,
                    &c.capitalized_price,
                    mappings,
                )?,
                capitalized_price: Price::from_height_source(
                    &name("capitalized_price"),
                    v,
                    &c.capitalized_price,
                    mappings,
                ),
                percentiles: id.select(&sources.per_dollar).prices.clone(),
            },
            min: price("cost_basis_min", id.select(&sources.min)),
            max: price("cost_basis_max", id.select(&sources.max)),
            supply_density: id.select(&sources.supply_density).series.clone(),
            capital_density: id.select(&sources.capital_density).series.clone(),
        })
    }

    pub(crate) fn compute(
        &mut self,
        from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.per_coin.avg.compute_ratio(from, spot, exit)?;
        self.per_dollar.avg.compute_ratio(from, spot, exit)
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 2] {
        [
            &mut self.per_coin.avg.relative.fixed.height,
            &mut self.per_dollar.avg.relative.fixed.height,
        ]
    }
}
