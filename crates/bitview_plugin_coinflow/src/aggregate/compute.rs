use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::WeightedCohortState;
use vecdb::{AnyStoredVec, WritableVec};

use super::Sources;

impl Sources {
    pub(crate) fn push(&mut self, states: &AgeAggregate<WeightedCohortState>) {
        for &id in AgeAggregateId::ALL {
            let state = id.select(states);
            id.select_mut(&mut self.mobile_supply)
                .push(state.weighted_supply);
            id.select_mut(&mut self.immobile_supply)
                .push(state.complement_supply);
            id.select_mut(&mut self.mobile_realized_cap)
                .push(state.weighted_cap);
            id.select_mut(&mut self.mobile_realized_price)
                .push(state.realized_price());
            id.select_mut(&mut self.mobile_capitalized_price)
                .push(state.capitalized_price.value());
            id.select_mut(&mut self.mobile_supply_in_loss_share)
                .push(state.supply_in_loss.value());
        }
    }

    pub(crate) fn vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let Self {
            mobile_supply,
            immobile_supply,
            mobile_realized_cap,
            mobile_realized_price,
            mobile_capitalized_price,
            mobile_supply_in_loss_share,
        } = self;
        mobile_supply
            .iter_mut()
            .map(|vec| vec as &mut dyn AnyStoredVec)
            .chain(
                immobile_supply
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                mobile_realized_cap
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                mobile_realized_price
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                mobile_capitalized_price
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
            .chain(
                mobile_supply_in_loss_share
                    .iter_mut()
                    .map(|vec| vec as &mut dyn AnyStoredVec),
            )
    }
}
