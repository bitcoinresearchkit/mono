use brk_types::{Cents, Sats, StoredF64};
use vecdb::unlikely;

pub(crate) mod coinflow;
pub(crate) mod cointime;
mod compute;
mod import;
mod vecs;

pub use vecs::Vecs;

pub const DB_NAME: &str = "frameworks";

#[derive(Clone, Copy, Default)]
pub(crate) struct WeightedRatio {
    numerator: f64,
    denominator: f64,
}

impl WeightedRatio {
    #[inline]
    pub(crate) fn add(&mut self, numerator: f64, denominator: f64, weight: f64) {
        if weight.is_finite() && weight > 0.0 {
            self.numerator += numerator * weight;
            self.denominator += denominator * weight;
        }
    }

    #[inline]
    pub(crate) fn merge(&mut self, other: Self) {
        self.numerator += other.numerator;
        self.denominator += other.denominator;
    }

    #[inline]
    pub(crate) fn value(&self) -> StoredF64 {
        if self.denominator > 0.0 {
            StoredF64::from((self.numerator / self.denominator).clamp(0.0, 1.0))
        } else {
            StoredF64::NAN
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct WeightedCohortContribution {
    pub weighted_supply: Sats,
    pub complement_supply: Sats,
    pub weighted_cap: Cents,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct WeightedCohortState {
    pub weighted_supply: Sats,
    pub complement_supply: Sats,
    pub weighted_cap: Cents,
    pub supply_in_loss: WeightedRatio,
}

impl WeightedCohortState {
    #[inline]
    pub(crate) fn split_supply(total: Sats, weight: StoredF64) -> (Sats, Sats) {
        (weight * total, (StoredF64::from(1.0) - weight) * total)
    }

    #[inline]
    pub(crate) fn add(
        &mut self,
        total_supply: Sats,
        loss_supply: Sats,
        total_cap: Cents,
        weight: StoredF64,
    ) -> WeightedCohortContribution {
        let (weighted_supply, complement_supply) = Self::split_supply(total_supply, weight);
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
        self
    }

    #[inline]
    pub(crate) fn realized_price(&self) -> Cents {
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

        let contribution = state.add(Sats::ZERO, Sats::ZERO, Cents::NAN, StoredF64::from(0.5));

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
            StoredF64::from(0.5),
        );

        assert!(state.weighted_cap.is_nan());
        assert!(state.realized_price().is_nan());
    }
}
