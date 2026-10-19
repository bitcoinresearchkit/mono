use bitview_primitives::{CostBasisByPercentile, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CostBasisMeans, CostBasisPerCoin, CostBasisPerDollar, Density, DensityVecs, IndexSources,
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
    pub per_coin: CostBasisPerCoin<M>,
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by each output's weighted USD value at creation.
    pub per_dollar: CostBasisPerDollar<M>,
    /// Share of the cohort's weighted supply with a creation price near spot.
    supply_density: DensityVecs<M>,
    /// Share of the cohort's weighted invested capital (satoshis times creation
    /// price) with a creation price near spot.
    capital_density: DensityVecs<M>,
    #[traversable(hidden)]
    per_coin_percentiles: PercentilesVecs<M>,
    #[traversable(hidden)]
    per_dollar_percentiles: PercentilesVecs<M>,
}

impl CostBasisVecs {
    /// `name` qualifies every id: `sth_awake` gives `sth_awake_cost_basis_per_coin_median`,
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
        let per_coin_percentiles = percentiles("cost_basis_per_coin")?;
        let per_dollar_percentiles = percentiles("cost_basis_per_dollar")?;
        let CostBasisMeans {
            per_coin,
            per_dollar,
        } = CostBasisMeans::import(
            db,
            metric,
            version,
            urpd_version,
            realized_price,
            capitalized_price,
            &per_coin_percentiles,
            &per_dollar_percentiles,
            indexes,
        )?;
        Ok(Self {
            per_coin,
            per_dollar,
            supply_density: density("supply_density")?,
            capital_density: density("capital_density")?,
            per_coin_percentiles,
            per_dollar_percentiles,
        })
    }

    /// Spot divided by each mean, stored.
    pub(crate) fn compute_ratios(
        &mut self,
        max_from: Height,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.per_coin.avg.compute_ratio(max_from, spot, exit)?;
        self.per_dollar.avg.compute_ratio(max_from, spot, exit)
    }

    pub(super) fn push(
        &mut self,
        cost_basis: &CostBasisByPercentile,
        supply_density: &Density<PartsPerMillion32>,
        capital_density: &Density<PartsPerMillion32>,
    ) {
        self.per_coin_percentiles.push(&cost_basis.per_coin);
        self.per_dollar_percentiles.push(&cost_basis.per_dollar);
        self.supply_density.push(supply_density);
        self.capital_density.push(capital_density);
    }

    pub(super) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.per_coin_percentiles
            .stored_vecs_mut()
            .chain(self.per_dollar_percentiles.stored_vecs_mut())
            .chain(self.supply_density.stored_vecs_mut())
            .chain(self.capital_density.stored_vecs_mut())
    }
}
