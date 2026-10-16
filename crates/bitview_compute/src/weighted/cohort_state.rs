use bitview_primitives::{BoundedRatio, Ratio64};
use brk_types::{Cents, Sats};

use super::{WeightedCostBasis, WeightedRatio, defined_weight};

#[derive(Clone, Copy, Default)]
pub struct WeightedCohortState {
    pub weighted_supply: Sats,
    pub complement_supply: Sats,
    pub supply_in_loss: WeightedRatio,
    pub cost_basis: WeightedCostBasis,
}

impl WeightedCohortState {
    /// The supply on each side of `weight`; an undefined weight (see [`defined_weight`]) puts it
    /// all on the unweighted side.
    #[inline]
    pub fn split_supply(total: Sats, weight: BoundedRatio) -> (Sats, Sats) {
        let weight = defined_weight(weight);
        (
            Ratio64::from(f64::from(weight)) * total,
            Ratio64::from(f64::from(weight.complement())) * total,
        )
    }

    #[inline]
    pub(crate) fn add(&mut self, total_supply: Sats, loss_supply: Sats, weight: BoundedRatio) {
        let (weighted_supply, complement_supply) = Self::split_supply(total_supply, weight);
        self.weighted_supply += weighted_supply;
        self.complement_supply += complement_supply;
        self.supply_in_loss.add(
            loss_supply.as_u128() as f64,
            total_supply.as_u128() as f64,
            f64::from(weight),
        );
    }

    #[inline]
    pub(crate) fn merged(mut self, other: Self) -> Self {
        self.weighted_supply += other.weighted_supply;
        self.complement_supply += other.complement_supply;
        self.supply_in_loss.merge(other.supply_in_loss);
        self.cost_basis.merge(other.cost_basis);
        self
    }

    /// The exact weighted cap over the weighted supply, rounded once.
    #[inline]
    pub fn realized_price(&self) -> Cents {
        self.cost_basis.realized_price(self.weighted_supply)
    }
}
