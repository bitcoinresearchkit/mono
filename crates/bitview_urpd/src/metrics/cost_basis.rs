use bitview_primitives::{CostBasisByPercentile, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    Density, DensityVecs, IndexSources, LazyPerBlock, PercentilesVecs, Price, PriceWithMvrv,
    PriceWithRatio,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, Database, Ident, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

/// A weighted cohort's cost basis: its mean creation prices from raw cost-basis moments, its
/// percentiles and densities from its URPD.
#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by the cohort's weighted satoshis.
    pub per_coin: PerCoin<M>,
    /// Creation prices (spot when each output was created) of the cohort's unspent outputs,
    /// weighted by each output's weighted USD value at creation.
    pub per_dollar: PerDollar<M>,
    /// Share of the cohort's weighted supply with a creation price near spot.
    supply_density: DensityVecs<M>,
    /// Share of the cohort's weighted invested capital (satoshis times creation
    /// price) with a creation price near spot.
    capital_density: DensityVecs<M>,
}

#[derive(Traversable)]
pub struct PerCoin<M: StorageMode = Rw> {
    /// The weighted mean: weighted realized cap divided by weighted supply; zero when the
    /// weighted supply is zero.
    pub avg: PriceWithMvrv<M>,
    /// Realized price: the weighted mean.
    pub realized_price: Price<LazyPerBlock<Cents>>,
    /// From the URPD, creation prices rounded to five significant digits.
    #[traversable(flatten)]
    pub percentiles: PercentilesVecs<M>,
    /// Median realized price: the weighted median, from the URPD.
    pub median_realized_price: Price<LazyPerBlock<Cents>>,
}

#[derive(Traversable)]
pub struct PerDollar<M: StorageMode = Rw> {
    /// The weighted mean: sum(weight × creation price² × sats) / sum(weight × creation price ×
    /// sats); zero when the weighted invested value is zero.
    pub avg: PriceWithRatio<M>,
    /// Capitalized price: the weighted mean.
    pub capitalized_price: Price<LazyPerBlock<Cents>>,
    /// From the URPD, creation prices rounded to five significant digits.
    #[traversable(flatten)]
    pub percentiles: PercentilesVecs<M>,
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
        let avg_version = version;
        let version = version + Version::new(3);
        let per_coin =
            PercentilesVecs::import(db, &format!("{name}_cost_basis_per_coin"), version, indexes)?;
        Ok(Self {
            per_coin: PerCoin {
                avg: PriceWithMvrv::import(
                    db,
                    &format!("{name}_cost_basis_per_coin_avg"),
                    &format!("{name}_mvrv"),
                    avg_version,
                    realized_price,
                    indexes,
                )?,
                realized_price: Price::from_height_source(
                    &format!("{name}_realized_price"),
                    avg_version,
                    realized_price,
                    indexes,
                ),
                median_realized_price: Price::from_lazy_cents_source::<Ident, Cents>(
                    &format!("{name}_median_realized_price"),
                    version,
                    &per_coin.median.cents,
                ),
                percentiles: per_coin,
            },
            per_dollar: PerDollar {
                avg: PriceWithRatio::import(
                    db,
                    &format!("{name}_cost_basis_per_dollar_avg"),
                    avg_version,
                    capitalized_price,
                    indexes,
                )?,
                capitalized_price: Price::from_height_source(
                    &format!("{name}_capitalized_price"),
                    avg_version,
                    capitalized_price,
                    indexes,
                ),
                percentiles: PercentilesVecs::import(
                    db,
                    &format!("{name}_cost_basis_per_dollar"),
                    version,
                    indexes,
                )?,
            },
            supply_density: DensityVecs::import(
                db,
                &format!("{name}_supply_density"),
                version,
                indexes,
            )?,
            capital_density: DensityVecs::import(
                db,
                &format!("{name}_capital_density"),
                version,
                indexes,
            )?,
        })
    }

    /// Spot divided by each mean, stored.
    pub fn compute_ratios(
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
        self.per_coin.percentiles.push(&cost_basis.per_coin);
        self.per_dollar.percentiles.push(&cost_basis.per_dollar);
        self.supply_density.push(supply_density);
        self.capital_density.push(capital_density);
    }

    pub(super) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.per_coin
            .percentiles
            .collect_vecs_mut()
            .into_iter()
            .chain(self.per_dollar.percentiles.collect_vecs_mut())
            .chain(self.supply_density.stored_vecs_mut())
            .chain(self.capital_density.stored_vecs_mut())
    }
}
