use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CostBasisByCoin, CostBasisByDollar, CostBasisMeans, Density, LazyPerBlock, LazyPercentPerBlock,
    PerBlock, Price,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableVec, Rw, StorageMode};

use super::CostBasisVecs;
use crate::columns::Columns;

/// Creation-price statistics of the cohort's unspent outputs.
#[derive(Traversable)]
pub struct CostBasis<M: StorageMode = Rw> {
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by satoshis.
    pub by_coin: CostBasisByCoin<M>,
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by each output's USD value at creation.
    pub by_dollar: CostBasisByDollar<M>,
    /// Lowest occupied bucket of the price grid among the cohort's unspent outputs: creation prices
    /// rounded to four significant digits of the price in cents (exact cents under $100, then steps
    /// of at most 0.1% up to $42,949,672.94, where higher prices cap). Undefined (null) without
    /// supply.
    #[traversable(lead = "Lowest cost basis: the lowest price any coin held last moved at.")]
    pub min: Price<LazyPerBlock<Cents>>,
    /// Highest occupied bucket of the price grid among the cohort's unspent outputs: creation
    /// prices rounded to four significant digits of the price in cents (exact cents under $100,
    /// then steps of at most 0.1% up to $42,949,672.94, where higher prices cap). Undefined (null)
    /// without supply.
    #[traversable(lead = "Highest cost basis: the highest price any coin held last moved at.")]
    pub max: Price<LazyPerBlock<Cents>>,
    /// Share of the cohort's unspent supply with a creation price near spot; undefined (null)
    /// without supply or a positive spot price.
    #[traversable(lead = "Supply near the price: the share of coins that last moved within 5% of the price of the day.")]
    pub supply_density: Density<LazyPercentPerBlock<PartsPerMillion32>>,
    /// Share of the cohort's invested capital (satoshis times creation price) with a creation
    /// price near spot; undefined (null) without capital or a positive spot price.
    #[traversable(lead = "Capital near the price: the share of what holders paid that was paid within 5% of the price of the day.")]
    pub capital_density: Density<LazyPercentPerBlock<PartsPerMillion32>>,
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
        let name = |metric: &str| id.metric_name(metric);
        let sources_v = CostBasisVecs::version(v);
        let price = |metric, source: &Price<PerBlock<Cents>>| {
            Price::from_height_source(&name(metric), sources_v, &source.cents.height, mappings)
        };
        let CostBasisMeans { by_coin, by_dollar } = CostBasisMeans::import(
            db,
            name,
            v,
            v,
            &c.price,
            &c.capitalized_price,
            id.select(&sources.by_coin),
            id.select(&sources.by_dollar),
            mappings,
        )?;
        Ok(Self {
            by_coin,
            by_dollar,
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
        self.by_coin.avg.compute_ratio(from, spot, exit)?;
        self.by_dollar.avg.compute_ratio(from, spot, exit)
    }
}
