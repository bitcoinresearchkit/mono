use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::WeightedCohortState;
use bitview_primitives::BoundedRatio;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, PcoVecValue, Rw, StorageMode, WritableVec};

/// Height-indexed sources of every aggregate cohort split by a weighted model: its weighted side
/// (awake, mobile) and the complement (dormant, immobile). The model owns the weights and the
/// public views.
#[derive(Traversable)]
pub struct WeightedCohortSources<M: StorageMode = Rw> {
    pub weighted_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub complement_supply: AgeAggregate<CachedSeries<Height, Sats, M>>,
    pub weighted_realized_cap: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub weighted_realized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub weighted_capitalized_price: AgeAggregate<CachedSeries<Height, Cents, M>>,
    pub weighted_supply_in_loss_share: AgeAggregate<CachedSeries<Height, BoundedRatio, M>>,
}

impl WeightedCohortSources {
    /// `weighted` and `complement` name the sides: `awake` gives `sth_awake_supply_sats`.
    pub fn import(
        db: &Database,
        weighted: &str,
        complement: &str,
        version: Version,
    ) -> Result<Self> {
        let weighted = |metric: &str| format!("{weighted}_{metric}");
        Ok(Self {
            weighted_supply: import_aggregate(db, &weighted("supply_sats"), version)?,
            complement_supply: import_aggregate(db, &format!("{complement}_supply_sats"), version)?,
            weighted_realized_cap: import_aggregate(db, &weighted("realized_cap_cents"), version)?,
            weighted_realized_price: import_aggregate(
                db,
                &weighted("realized_price_cents"),
                version,
            )?,
            weighted_capitalized_price: import_aggregate(
                db,
                &weighted("capitalized_price_cents"),
                version,
            )?,
            weighted_supply_in_loss_share: import_aggregate(
                db,
                &weighted("supply_in_loss_share_bounded"),
                version,
            )?,
        })
    }

    pub fn push(&mut self, states: &AgeAggregate<WeightedCohortState>) {
        for &id in AgeAggregateId::ALL {
            let state = id.select(states);
            id.select_mut(&mut self.weighted_supply)
                .push(state.weighted_supply);
            id.select_mut(&mut self.complement_supply)
                .push(state.complement_supply);
            id.select_mut(&mut self.weighted_realized_cap)
                .push(state.weighted_cap);
            id.select_mut(&mut self.weighted_realized_price)
                .push(state.realized_price());
            id.select_mut(&mut self.weighted_capitalized_price)
                .push(state.capitalized_price.value());
            id.select_mut(&mut self.weighted_supply_in_loss_share)
                .push(state.supply_in_loss.value());
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            weighted_supply,
            complement_supply,
            weighted_realized_cap,
            weighted_realized_price,
            weighted_capitalized_price,
            weighted_supply_in_loss_share,
        } = self;
        fn stored<T: PcoVecValue>(
            vecs: &mut AgeAggregate<CachedSeries<Height, T>>,
        ) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
            vecs.iter_mut().map(|vec| vec as &mut dyn AnyStoredVec)
        }
        stored(weighted_supply)
            .chain(stored(complement_supply))
            .chain(stored(weighted_realized_cap))
            .chain(stored(weighted_realized_price))
            .chain(stored(weighted_capitalized_price))
            .chain(stored(weighted_supply_in_loss_share))
    }
}

fn import_aggregate<T: PcoVecValue>(
    db: &Database,
    metric: &str,
    version: Version,
) -> Result<AgeAggregate<CachedSeries<Height, T>>> {
    AgeAggregate::try_from_fn(|id| import_cached(db, &id.metric_name(metric), version))
}
