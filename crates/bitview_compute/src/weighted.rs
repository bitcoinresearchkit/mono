use std::ops::AddAssign;

use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_primitives::{BoundedRatio, CentsSquaredSats};
use brk_types::{Cents, CentsSats, Sats};

/// One age range's additive amounts at one height, as the age plugin stores them: what its weight
/// scales.
#[derive(Clone, Copy)]
pub(crate) struct BandAmounts {
    pub(crate) supply: Sats,
    pub(crate) loss_supply: Sats,
    pub(crate) cap: CentsSats,
    pub(crate) capitalized_cap: CentsSquaredSats,
    pub(crate) unrealized_profit: Cents,
    pub(crate) unrealized_loss: Cents,
    pub(crate) capital_in_profit: Cents,
    pub(crate) capital_in_loss: Cents,
}

/// Exact moments of disjoint age ranges, each under its own weight: their total supply, and the
/// weighted sums of their supply, supply in loss, cap (creation price times satoshis),
/// capitalized cap (squared creation price times satoshis), unrealized profit and loss, and
/// capital in profit and in loss. Every published value derives from the unrounded moments,
/// rounded once: supplies to the nearest satoshi (the complement is the rest of the total), caps,
/// amounts and prices to the nearest cent. A weight of one adds each range's amount as it is, so
/// the weighted sums at one are the plain sums of the ranges. Unweighted moment totals fit in
/// u128.
#[derive(Clone, Copy, Default)]
pub struct WeightedCohortState {
    total_supply: Sats,
    supply: WeightedSum,
    loss_supply: WeightedSum,
    cap: WeightedSum,
    capitalized_cap: WeightedSum,
    unrealized_profit: WeightedSum,
    unrealized_loss: WeightedSum,
    capital_in_profit: WeightedSum,
    capital_in_loss: WeightedSum,
}

impl WeightedCohortState {
    /// One age range's moments. A weight its range's history cannot define yet (no coin-days in
    /// the range) moves nothing: the coins stay on the unweighted side (dormant, immobile). The
    /// published weight itself stays undefined.
    #[inline]
    pub(crate) fn new(band: BandAmounts, weight: BoundedRatio) -> Self {
        let weight = defined(weight);
        let weighted = |value: u128| WeightedSum::new(value, weight);
        let cents = |value: Cents| {
            debug_assert!(!value.is_nan(), "undefined band amount");
            weighted(u128::from(value.inner()))
        };
        Self {
            total_supply: band.supply,
            supply: weighted(band.supply.as_u128()),
            loss_supply: weighted(band.loss_supply.as_u128()),
            cap: weighted(band.cap.inner()),
            capitalized_cap: weighted(band.capitalized_cap.inner()),
            unrealized_profit: cents(band.unrealized_profit),
            unrealized_loss: cents(band.unrealized_loss),
            capital_in_profit: cents(band.capital_in_profit),
            capital_in_loss: cents(band.capital_in_loss),
        }
    }

    /// One range's supply on each side of `weight`, as its aggregates publish theirs.
    #[inline]
    pub fn split_supply(total: Sats, weight: BoundedRatio) -> (Sats, Sats) {
        let weighted = round_sats(WeightedSum::new(total.as_u128(), defined(weight)));
        (weighted, total - weighted)
    }

    pub fn weighted_supply(&self) -> Sats {
        round_sats(self.supply)
    }

    pub fn complement_supply(&self) -> Sats {
        self.total_supply - self.weighted_supply()
    }

    /// The weighted supply in loss; at most the weighted supply, both rounded from exact sums.
    pub fn supply_in_loss(&self) -> Sats {
        round_sats(self.loss_supply)
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

    pub fn unrealized_profit(&self) -> Cents {
        Cents::from(self.unrealized_profit.div_round(1))
    }

    pub fn unrealized_loss(&self) -> Cents {
        Cents::from(self.unrealized_loss.div_round(1))
    }

    pub fn capital_in_profit(&self) -> Cents {
        Cents::from(self.capital_in_profit.div_round(1))
    }

    pub fn capital_in_loss(&self) -> Cents {
        Cents::from(self.capital_in_loss.div_round(1))
    }
}

/// An undefined weight moves nothing (see [`WeightedCohortState::new`]).
#[inline]
fn defined(weight: BoundedRatio) -> BoundedRatio {
    if weight.is_nan() { BoundedRatio::ZERO } else { weight }
}

#[inline]
fn round_sats(sum: WeightedSum) -> Sats {
    Sats::new(sum.div_round(1) as u64)
}

impl AddAssign for WeightedCohortState {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.total_supply += rhs.total_supply;
        self.supply += rhs.supply;
        self.loss_supply += rhs.loss_supply;
        self.cap += rhs.cap;
        self.capitalized_cap += rhs.capitalized_cap;
        self.unrealized_profit += rhs.unrealized_profit;
        self.unrealized_loss += rhs.unrealized_loss;
        self.capital_in_profit += rhs.capital_in_profit;
        self.capital_in_loss += rhs.capital_in_loss;
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
