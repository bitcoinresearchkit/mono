use bitview_primitives::{BoundedRatio, Ratio64};
use brk_types::{Cents, Sats};
use vecdb::unlikely;

use super::{WeightedCapitalizedPrice, WeightedCohortContribution, WeightedRatio};

#[derive(Clone, Copy, Default)]
pub struct WeightedCohortState {
    pub weighted_supply: Sats,
    pub complement_supply: Sats,
    pub weighted_cap: Cents,
    pub supply_in_loss: WeightedRatio,
    pub capitalized_price: WeightedCapitalizedPrice,
}

impl WeightedCohortState {
    #[inline]
    pub fn split_supply(total: Sats, weight: BoundedRatio) -> (Sats, Sats) {
        (
            Ratio64::from(f64::from(weight)) * total,
            Ratio64::from(f64::from(weight.complement())) * total,
        )
    }

    #[inline]
    pub(crate) fn add(
        &mut self,
        total_supply: Sats,
        loss_supply: Sats,
        total_cap: Cents,
        weight: BoundedRatio,
    ) -> WeightedCohortContribution {
        let (weighted_supply, complement_supply) = Self::split_supply(total_supply, weight);
        let weight = Ratio64::from(f64::from(weight));
        let contribution = WeightedCohortContribution {
            weighted_supply,
            complement_supply,
            weighted_cap: if total_supply.is_zero() {
                Cents::ZERO
            } else {
                weight * total_cap
            },
        };

        self.weighted_supply += contribution.weighted_supply;
        self.complement_supply += contribution.complement_supply;
        self.weighted_cap += contribution.weighted_cap;
        self.supply_in_loss.add(
            loss_supply.as_u128() as f64,
            total_supply.as_u128() as f64,
            f64::from(weight),
        );

        contribution
    }

    #[inline]
    pub(crate) fn merged(mut self, other: Self) -> Self {
        self.weighted_supply += other.weighted_supply;
        self.complement_supply += other.complement_supply;
        self.weighted_cap += other.weighted_cap;
        self.supply_in_loss.merge(other.supply_in_loss);
        self.capitalized_price.merge(other.capitalized_price);
        self
    }

    #[inline]
    pub fn realized_price(&self) -> Cents {
        if unlikely(self.weighted_cap.is_nan()) {
            return Cents::NAN;
        }

        (self.weighted_cap.as_u128() * Sats::ONE_BTC_U128)
            .checked_div(self.weighted_supply.as_u128())
            .map(Cents::from)
            .unwrap_or(Cents::ZERO)
    }
}
