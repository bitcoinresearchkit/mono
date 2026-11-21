use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::WeightedCohortState;
use bitview_primitives::BoundedRatio;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, import_cached};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{Database, PcoVecValue, Rw, StorageMode, WritableVec};

/// Height-indexed sources of every aggregate cohort split by a weighted model: its weighted side
/// (awake, mobile) and the complement (dormant, immobile). The model owns the weights and the
/// public views.
#[derive(Traversable)]
pub struct WeightedCohortSources<M: StorageMode = Rw> {
    pub weighted_supply: AgeAggregate<CachedSeries<Sats, M>>,
    pub complement_supply: AgeAggregate<CachedSeries<Sats, M>>,
    pub weighted_realized_cap: AgeAggregate<CachedSeries<Cents, M>>,
    pub weighted_realized_price: AgeAggregate<CachedSeries<Cents, M>>,
    pub weighted_capitalized_price: AgeAggregate<CachedSeries<Cents, M>>,
    pub weighted_supply_in_loss_share: AgeAggregate<CachedSeries<BoundedRatio, M>>,
}

impl WeightedCohortSources {
    /// `weighted` and `complement` name the sides: `awake` gives `sth_awake_supply_sats`.
    pub fn import(
        db: &Database,
        weighted: &str,
        complement: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let weighted = |metric: &str| format!("{weighted}_{metric}");
        Ok(Self {
            weighted_supply: import_aggregate(db, &weighted("supply_sats"), version, indexes)?,
            complement_supply: import_aggregate(
                db,
                &format!("{complement}_supply_sats"),
                version,
                indexes,
            )?,
            weighted_realized_cap: import_aggregate(
                db,
                &weighted("realized_cap_cents"),
                version,
                indexes,
            )?,
            weighted_realized_price: import_aggregate(
                db,
                &weighted("realized_price_cents"),
                version,
                indexes,
            )?,
            weighted_capitalized_price: import_aggregate(
                db,
                &weighted("capitalized_price_cents"),
                version,
                indexes,
            )?,
            weighted_supply_in_loss_share: import_aggregate(
                db,
                &weighted("supply_in_loss_share_bounded"),
                version,
                indexes,
            )?,
        })
    }

    pub fn push(&mut self, states: &AgeAggregate<WeightedCohortState>) {
        for &id in AgeAggregateId::ALL {
            let state = id.select(states);
            id.select_mut(&mut self.weighted_supply)
                .push(state.weighted_supply());
            id.select_mut(&mut self.complement_supply)
                .push(state.complement_supply());
            id.select_mut(&mut self.weighted_realized_cap)
                .push(state.realized_cap());
            id.select_mut(&mut self.weighted_realized_price)
                .push(state.realized_price());
            id.select_mut(&mut self.weighted_capitalized_price)
                .push(state.capitalized_price());
            id.select_mut(&mut self.weighted_supply_in_loss_share)
                .push(state.supply_in_loss_share());
        }
    }
}

fn import_aggregate<T: PcoVecValue>(
    db: &Database,
    metric: &str,
    version: Version,
    indexes: &IndexSources,
) -> Result<AgeAggregate<CachedSeries<T>>> {
    AgeAggregate::try_from_fn(|id| import_cached(db, &id.metric_name(metric), version, indexes))
}
