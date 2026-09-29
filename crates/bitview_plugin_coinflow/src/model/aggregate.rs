use bitview_cohort::{AgeRange, AgeRangeId};
use bitview_compute::{WeightedCohortContribution, WeightedCohortState, WeightedRatio};
use brk_types::{BoundedRatio, Cents, Sats};

use crate::{HorizonId, Horizons};

#[derive(Clone, Copy)]
pub(crate) struct AggregateState {
    pub(crate) weighted: WeightedCohortState,
    pub(crate) horizon_supply_in_loss: Horizons<WeightedRatio>,
}

impl Default for AggregateState {
    fn default() -> Self {
        Self {
            weighted: WeightedCohortState::default(),
            horizon_supply_in_loss: HorizonId::from_fn(|_| WeightedRatio::default()),
        }
    }
}

impl AggregateState {
    pub(super) fn add(
        &mut self,
        total_supply: Sats,
        loss_supply: Sats,
        total_cap: Cents,
        mobility: BoundedRatio,
        horizon_mobilities: &Horizons<AgeRange<f64>>,
        age: AgeRangeId,
    ) -> WeightedCohortContribution {
        let contribution = self
            .weighted
            .add(total_supply, loss_supply, total_cap, mobility);
        let total = total_supply.as_u128() as f64;
        let loss = loss_supply.as_u128() as f64;
        for horizon in HorizonId::ALL {
            let ratio = horizon.select_mut(&mut self.horizon_supply_in_loss);
            let weights = horizon.select(horizon_mobilities);
            ratio.add(loss, total, *age.select(weights));
        }
        contribution
    }

    pub(crate) fn merged(mut self, other: Self) -> Self {
        self.weighted = self.weighted.merged(other.weighted);
        self.merge_horizons(&other);
        self
    }

    pub(super) fn merge_horizons(&mut self, other: &Self) {
        for horizon in HorizonId::ALL {
            horizon
                .select_mut(&mut self.horizon_supply_in_loss)
                .merge(*horizon.select(&other.horizon_supply_in_loss));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_aggregates_merge_into_all() {
        let horizons = HorizonId::from_fn(|_| AgeRange::from_fn(|_| 0.25));
        let mut direct = AggregateState::default();
        direct.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            BoundedRatio::from(0.3),
            &horizons,
            AgeRangeId::Under1H,
        );
        direct.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            BoundedRatio::from(0.4),
            &horizons,
            AgeRangeId::From1HTo1D,
        );

        let mut sth = AggregateState::default();
        sth.add(
            Sats::from(100_u64),
            Sats::from(20_u64),
            Cents::from(1_000_u64),
            BoundedRatio::from(0.3),
            &horizons,
            AgeRangeId::Under1H,
        );
        let mut lth = AggregateState::default();
        lth.add(
            Sats::from(200_u64),
            Sats::from(50_u64),
            Cents::from(3_000_u64),
            BoundedRatio::from(0.4),
            &horizons,
            AgeRangeId::From1HTo1D,
        );
        let merged = sth.merged(lth);

        assert_eq!(
            merged.weighted.weighted_supply,
            direct.weighted.weighted_supply
        );
        assert_eq!(
            merged.weighted.complement_supply,
            direct.weighted.complement_supply
        );
        assert_eq!(merged.weighted.weighted_cap, direct.weighted.weighted_cap);
        assert_eq!(
            merged.weighted.supply_in_loss.value(),
            direct.weighted.supply_in_loss.value()
        );
        for horizon in HorizonId::ALL {
            assert_eq!(
                horizon.select(&merged.horizon_supply_in_loss).value(),
                horizon.select(&direct.horizon_supply_in_loss).value(),
            );
        }
    }
}
