use bitview_primitives::{CostBasisByPercentile, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{Density, DensityVecs, IndexSources, PercentilesVecs};
use brk_error::Result;
use brk_types::Version;
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

/// A weighted cohort's cost basis from its rounded URPD creation prices.
#[derive(Traversable)]
pub struct CostBasisVecs<M: StorageMode = Rw> {
    /// Creation-price percentiles weighted by the cohort's weighted satoshis.
    pub per_coin: PercentilesVecs<M>,
    /// Creation-price percentiles weighted by each output's weighted USD value
    /// at creation.
    pub per_dollar: PercentilesVecs<M>,
    /// Share of the cohort's weighted supply with a creation price near spot.
    supply_density: DensityVecs<M>,
    /// Share of the cohort's weighted invested capital (satoshis times creation
    /// price) with a creation price near spot.
    capital_density: DensityVecs<M>,
}

impl CostBasisVecs {
    /// `name` qualifies every id: `sth_awake` gives `sth_awake_cost_basis_per_coin_median`
    /// and `sth_awake_supply_density`.
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let version = version + Version::new(3);
        Ok(Self {
            per_coin: PercentilesVecs::import(
                db,
                &format!("{name}_cost_basis_per_coin"),
                version,
                indexes,
            )?,
            per_dollar: PercentilesVecs::import(
                db,
                &format!("{name}_cost_basis_per_dollar"),
                version,
                indexes,
            )?,
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

    pub(super) fn push(
        &mut self,
        cost_basis: &CostBasisByPercentile,
        supply_density: &Density<PartsPerMillion32>,
        capital_density: &Density<PartsPerMillion32>,
    ) {
        self.per_coin.push(&cost_basis.per_coin);
        self.per_dollar.push(&cost_basis.per_dollar);
        self.supply_density.push(supply_density);
        self.capital_density.push(capital_density);
    }

    pub(super) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.per_coin
            .collect_vecs_mut()
            .into_iter()
            .chain(self.per_dollar.collect_vecs_mut())
            .chain(self.supply_density.stored_vecs_mut())
            .chain(self.capital_density.stored_vecs_mut())
    }
}
