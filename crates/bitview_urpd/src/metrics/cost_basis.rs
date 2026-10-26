use bitview_primitives::{CostBasisByPercentile, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CostBasisMeans, CostBasisByCoin, CostBasisByDollar, Density, DensityVecs, IndexSources,
    PercentilesVecs,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

/// A weighted cohort's cost basis: its mean creation prices from raw cost-basis moments, its
/// percentiles and densities from its URPD.
#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by the cohort's weighted satoshis.
    pub by_coin: CostBasisByCoin<M>,
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by each output's weighted USD value at creation.
    pub by_dollar: CostBasisByDollar<M>,
    /// Share of the cohort's weighted supply with a creation price near spot; undefined (null)
    /// without weighted supply or a positive spot price.
    #[traversable(lead = "Supply near the price: the share of coins that last moved within 5% of the price of the day.")]
    supply_density: DensityVecs<M>,
    /// Share of the cohort's weighted invested capital (satoshis times creation
    /// price) with a creation price near spot; undefined (null) without weighted capital or a
    /// positive spot price.
    #[traversable(lead = "Capital near the price: the share of what holders paid that was paid within 5% of the price of the day.")]
    capital_density: DensityVecs<M>,
    #[traversable(hidden)]
    by_coin_percentiles: PercentilesVecs<M>,
    #[traversable(hidden)]
    by_dollar_percentiles: PercentilesVecs<M>,
}

impl CostBasisVecs {
    /// `name` qualifies every id: `sth_awake` gives `sth_awake_cost_basis_by_coin_median`,
    /// `sth_awake_realized_price` and `sth_awake_supply_density`. The means are lazy over
    /// `realized_price` and `capitalized_price`.
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        realized_price: &impl ReadableCloneableVec<Height, Cents>,
        capitalized_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let metric = |metric: &str| format!("{name}_{metric}");
        let urpd_version = version + Version::new(3);
        // Exact rational band edges.
        let density_version = urpd_version + Version::ONE;
        let percentiles = |metric: &str| {
            PercentilesVecs::import(db, &format!("{name}_{metric}"), urpd_version, indexes)
        };
        let density = |metric: &str| {
            DensityVecs::import(db, &format!("{name}_{metric}"), density_version, indexes)
        };
        let by_coin_percentiles = percentiles("cost_basis_by_coin")?;
        let by_dollar_percentiles = percentiles("cost_basis_by_dollar")?;
        let CostBasisMeans {
            by_coin,
            by_dollar,
        } = CostBasisMeans::import(
            db,
            metric,
            version,
            urpd_version,
            realized_price,
            capitalized_price,
            &by_coin_percentiles,
            &by_dollar_percentiles,
            indexes,
        )?;
        Ok(Self {
            by_coin,
            by_dollar,
            supply_density: density("supply_density")?,
            capital_density: density("capital_density")?,
            by_coin_percentiles,
            by_dollar_percentiles,
        })
    }

    /// Spot divided by each mean, stored.
    pub(crate) fn compute_ratios(
        &mut self,
        max_from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.by_coin.avg.compute_ratio(max_from, spot, exit)?;
        self.by_dollar.avg.compute_ratio(max_from, spot, exit)
    }

    pub(super) fn push(
        &mut self,
        cost_basis: &CostBasisByPercentile,
        supply_density: &Density<PartsPerMillion32>,
        capital_density: &Density<PartsPerMillion32>,
    ) {
        self.by_coin_percentiles.push(&cost_basis.by_coin);
        self.by_dollar_percentiles.push(&cost_basis.by_dollar);
        self.supply_density.push(supply_density);
        self.capital_density.push(capital_density);
    }

    pub(super) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.by_coin_percentiles
            .stored_vecs_mut()
            .chain(self.by_dollar_percentiles.stored_vecs_mut())
            .chain(self.supply_density.stored_vecs_mut())
            .chain(self.capital_density.stored_vecs_mut())
    }
}
