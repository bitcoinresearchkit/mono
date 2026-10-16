use std::ops::AddAssign;

use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_primitives::{BoundedRatio, CentsSquaredSats};
use brk_types::{Cents, CentsSats, Sats};

/// Exact moments of disjoint age ranges, each under its own weight: their total supply, and
/// their weighted supply, supply in loss, cap (creation price times satoshis) and capitalized
/// cap (squared creation price times satoshis). Every published value derives from the unrounded
/// moments, rounded once: the weighted supply to the nearest satoshi (the complement is the rest
/// of the total), the realized cap and prices to the nearest cent. Unweighted moment totals fit
/// in u128.
#[derive(Clone, Copy, Default)]
pub struct WeightedCohortState {
    total_supply: Sats,
    supply: WeightedSum,
    loss_supply: WeightedSum,
    cap: WeightedSum,
    capitalized_cap: WeightedSum,
}

impl WeightedCohortState {
    /// One age range's moments. A weight its range's history cannot define yet (no coin-days in
    /// the range) moves nothing: the coins stay on the unweighted side (dormant, immobile). The
    /// published weight itself stays undefined.
    #[inline]
    pub(crate) fn new(
        total_supply: Sats,
        loss_supply: Sats,
        cap: CentsSats,
        capitalized_cap: CentsSquaredSats,
        weight: BoundedRatio,
    ) -> Self {
        let weight = if weight.is_nan() {
            BoundedRatio::ZERO
        } else {
            weight
        };
        let weighted = |value: u128| WeightedSum::new(value, weight);
        Self {
            total_supply,
            supply: weighted(total_supply.as_u128()),
            loss_supply: weighted(loss_supply.as_u128()),
            cap: weighted(cap.inner()),
            capitalized_cap: weighted(capitalized_cap.inner()),
        }
    }

    /// One range's supply on each side of `weight`, as its aggregates publish theirs.
    #[inline]
    pub fn split_supply(total: Sats, weight: BoundedRatio) -> (Sats, Sats) {
        let state = Self::new(
            total,
            Sats::ZERO,
            CentsSats::ZERO,
            CentsSquaredSats::default(),
            weight,
        );
        (state.weighted_supply(), state.complement_supply())
    }

    pub fn weighted_supply(&self) -> Sats {
        Sats::new(self.supply.div_round(1) as u64)
    }

    pub fn complement_supply(&self) -> Sats {
        self.total_supply - self.weighted_supply()
    }

    pub fn realized_cap(&self) -> Cents {
        Cents::from(self.cap.div_round(Sats::ONE_BTC_U128))
    }

    /// The cap per weighted supply; undefined (NaN) without weighted supply.
    pub fn realized_price(&self) -> Cents {
        self.cap.div_round_cents(self.supply)
    }

    /// The capitalized cap per cap; undefined (NaN) without cap.
    pub fn capitalized_price(&self) -> Cents {
        self.capitalized_cap.div_round_cents(self.cap)
    }

    /// The weighted supply in loss per weighted supply; undefined (NaN) without weighted supply.
    pub fn supply_in_loss_share(&self) -> BoundedRatio {
        BoundedRatio::from(self.loss_supply.ratio(self.supply))
    }
}

impl AddAssign for WeightedCohortState {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.total_supply += rhs.total_supply;
        self.supply += rhs.supply;
        self.loss_supply += rhs.loss_supply;
        self.cap += rhs.cap;
        self.capitalized_cap += rhs.capitalized_cap;
    }
}

/// Merges each age range's weighted state into every age aggregate containing it.
pub fn weighted_age_aggregates(
    mut range: impl FnMut(AgeRangeId) -> WeightedCohortState,
) -> AgeAggregate<WeightedCohortState> {
    let mut aggregates = AgeAggregate::<WeightedCohortState>::default();
    for &id in AgeRangeId::ALL {
        let state = range(id);
        for aggregate in aggregates.containing_mut(id) {
            *aggregate += state;
        }
    }
    aggregates
}

/// Exact nonnegative sum in units of 1 / BoundedRatio::SCALE. Splitting the integer and fractional
/// parts avoids a potentially overflowing u128 * u32.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct WeightedSum {
    whole: u128,
    remainder: u64,
}

impl WeightedSum {
    const SCALE: u128 = BoundedRatio::SCALE as u128;

    #[inline]
    fn new(value: u128, weight: BoundedRatio) -> Self {
        debug_assert!(!weight.is_nan(), "undefined weight");
        let weight = u128::from(weight.inner());
        let fractional = (value % Self::SCALE) * weight;
        Self {
            whole: (value / Self::SCALE) * weight + fractional / Self::SCALE,
            remainder: (fractional % Self::SCALE) as u64,
        }
    }

    /// The sum over `divisor`, rounded to the nearest integer (halves up, as `CentsSats` rounds).
    #[inline]
    fn div_round(self, divisor: u128) -> u128 {
        let (quotient, rest) = (self.whole / divisor, self.whole % divisor);
        // Up when the rest, `rest + remainder / SCALE`, is at least half the divisor. Both sides
        // stay far below u128::MAX: `divisor` is at most one BTC in satoshis.
        let doubled_rest = 2 * (rest * Self::SCALE + u128::from(self.remainder));
        quotient + u128::from(doubled_rest >= divisor * Self::SCALE)
    }

    /// This sum over `divisor` (NaN when both are zero, infinite when only `divisor` is), within a
    /// relative 2^-50 of the exact quotient.
    #[inline]
    fn ratio(self, divisor: Self) -> f64 {
        f64::from(self) / f64::from(divisor)
    }

    /// This sum over `divisor`, rounded to the nearest cent (halves up); undefined (NaN) without
    /// `divisor`. The f64 quotient lands within a cent of it; the half-cent boundaries on either
    /// side of that one settle it exactly. Weighted means of prices stay under `Cents::MAX_FINITE`.
    #[inline]
    fn div_round_cents(self, divisor: Self) -> Cents {
        let estimate = Cents::from(self.ratio(divisor));
        let Some(cents) = estimate.finite_inner() else {
            return estimate;
        };
        let doubled = self.multiplied(2);
        // Whether this sum over `divisor` reaches `halves` / 2.
        let reaches = |halves: u128| {
            divisor
                .multiplied(halves)
                .zip(doubled)
                .is_some_and(|(boundary, doubled)| boundary <= doubled)
        };
        let cents = u128::from(cents);
        Cents::from(if cents > 0 && !reaches(2 * cents - 1) {
            cents - 1
        } else if reaches(2 * cents + 1) {
            cents + 1
        } else {
            cents
        })
    }

    /// This sum times `factor`; None on overflow.
    #[inline]
    fn multiplied(self, factor: u128) -> Option<Self> {
        let fractional = u128::from(self.remainder) * factor;
        Some(Self {
            whole: self
                .whole
                .checked_mul(factor)?
                .checked_add(fractional / Self::SCALE)?,
            remainder: (fractional % Self::SCALE) as u64,
        })
    }
}

impl From<WeightedSum> for f64 {
    #[inline]
    fn from(sum: WeightedSum) -> Self {
        sum.whole as f64 + sum.remainder as f64 / WeightedSum::SCALE as f64
    }
}

impl AddAssign for WeightedSum {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        let remainder = self.remainder + rhs.remainder;
        self.whole += rhs.whole + u128::from(remainder / u64::from(BoundedRatio::SCALE));
        self.remainder = remainder % u64::from(BoundedRatio::SCALE);
    }
}
