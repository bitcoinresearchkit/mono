use brk_types::{BoundedRatio, Cents, Sats, StoredF64};
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
            StoredF64::from(f64::from(weight)) * total,
            StoredF64::from(f64::from(weight.complement())) * total,
        )
    }

    #[inline]
    pub fn add(
        &mut self,
        total_supply: Sats,
        loss_supply: Sats,
        total_cap: Cents,
        weight: BoundedRatio,
    ) -> WeightedCohortContribution {
        let (weighted_supply, complement_supply) = Self::split_supply(total_supply, weight);
        let weight = StoredF64::from(f64::from(weight));
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
    pub fn merged(mut self, other: Self) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_nan_cap_contributes_zero() {
        let mut state = WeightedCohortState::default();

        let contribution = state.add(Sats::ZERO, Sats::ZERO, Cents::NAN, BoundedRatio::from(0.5));

        assert_eq!(contribution.weighted_cap, Cents::ZERO);
        assert_eq!(state.weighted_cap, Cents::ZERO);
        assert_eq!(state.realized_price(), Cents::ZERO);
    }

    #[test]
    fn nonempty_nan_cap_remains_nan() {
        let mut state = WeightedCohortState::default();

        state.add(
            Sats::from(100_u64),
            Sats::ZERO,
            Cents::NAN,
            BoundedRatio::from(0.5),
        );

        assert!(state.weighted_cap.is_nan());
        assert!(state.realized_price().is_nan());
    }

    #[test]
    fn bounded_weights_use_the_encoded_complement() {
        let total = Sats::from(123_456_789_u64);
        let total_cap = Cents::from(987_654_321_u64);
        for value in [0.0, 1.0 / 3.0, 0.321, 1.0] {
            let weight = BoundedRatio::from(value);
            let split = WeightedCohortState::split_supply(total, weight);
            assert_eq!(split.0, StoredF64::from(f64::from(weight)) * total);
            assert_eq!(
                split.1,
                StoredF64::from(f64::from(weight.complement())) * total
            );
            assert!(split.0 + split.1 <= total);
            assert!(total - split.0 - split.1 <= Sats::from(1_u64));
            let mut state = WeightedCohortState::default();
            let contribution = state.add(total, Sats::ZERO, total_cap, weight);
            assert_eq!(contribution.weighted_supply, split.0);
            assert_eq!(contribution.complement_supply, split.1);
            assert!(state.weighted_cap <= total_cap);
        }
    }
}
