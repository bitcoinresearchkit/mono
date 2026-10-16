use bitview_primitives::{BoundedRatio, CentsSquaredSats};
use brk_types::{Cents, CentsSats, Sats};

/// Weighted raw cost-basis moments of disjoint cohorts, both under the same encoded weight: the
/// cap (creation price times satoshis) and the capitalized cap (squared creation price times
/// satoshis). Fractional raw units are retained; each value derived from them is rounded once,
/// to the nearest cent. Inputs are disjoint cohorts whose unweighted moment totals fit in u128.
#[derive(Clone, Copy, Default)]
pub struct WeightedCostBasis {
    cap: WeightedSum,
    capitalized_cap: WeightedSum,
}

impl WeightedCostBasis {
    pub(crate) fn add(
        &mut self,
        cap: CentsSats,
        capitalized_cap: CentsSquaredSats,
        weight: BoundedRatio,
    ) {
        debug_assert!(!weight.is_nan(), "undefined weight");
        if cap == CentsSats::ZERO || weight == BoundedRatio::ZERO {
            return;
        }
        self.cap.add(cap.inner(), weight);
        self.capitalized_cap.add(capitalized_cap.inner(), weight);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.cap.merge(other.cap);
        self.capitalized_cap.merge(other.capitalized_cap);
    }

    /// The weighted realized cap.
    pub fn realized_cap(&self) -> Cents {
        Cents::from(self.cap.div_round(Sats::ONE_BTC_U128))
    }

    /// The weighted realized cap per weighted `supply`; undefined (NaN) without supply.
    pub(crate) fn realized_price(&self, supply: Sats) -> Cents {
        if supply.is_zero() {
            return Cents::NAN;
        }
        Cents::from(self.cap.div_round(supply.as_u128()))
    }

    /// The capitalized cap per cap; undefined (NaN) without cap.
    pub fn capitalized_price(&self) -> Cents {
        if self.cap == WeightedSum::default() {
            return Cents::NAN;
        }
        let floor = self.floor();
        // Up when the remainder is at least half the cap:
        // (2 × floor + 1) × cap <= 2 × capitalized cap.
        let up = floor
            .checked_mul(2)
            .and_then(|doubled| self.cap.multiplied(doubled + 1))
            .zip(self.capitalized_cap.multiplied(2))
            .is_some_and(|(half_up, numerator)| half_up <= numerator);
        Cents::new(floor + u64::from(up))
    }

    fn floor(&self) -> u64 {
        // Find floor(capitalized cap / cap) without overflowing either raw
        // moment by multiplying it by the weight scale or the candidate price.
        let mut low = 0;
        let mut high = self
            .capitalized_cap
            .whole
            .checked_div(self.cap.whole)
            .unwrap_or(Cents::MAX_FINITE.as_u128())
            .min(Cents::MAX_FINITE.as_u128()) as u64;
        // With substantial invested value the integer-parts quotient is
        // usually already exact, so avoid searching in the common case.
        if self
            .cap
            .multiplied(high)
            .is_some_and(|value| value <= self.capitalized_cap)
        {
            return high;
        }
        while low < high {
            let mid = low + (high - low) / 2 + (high - low) % 2;
            if self
                .cap
                .multiplied(mid)
                .is_some_and(|value| value <= self.capitalized_cap)
            {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        low
    }
}

/// Exact nonnegative sum in units of 1 / BoundedRatio::SCALE. Splitting the
/// integer and fractional parts avoids a potentially overflowing u128 * u32.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct WeightedSum {
    whole: u128,
    remainder: u64,
}

impl WeightedSum {
    const SCALE: u128 = BoundedRatio::SCALE as u128;

    fn add(&mut self, value: u128, weight: BoundedRatio) {
        let weight = u128::from(weight.inner());
        let fractional = (value % Self::SCALE) * weight;
        self.merge(Self {
            whole: (value / Self::SCALE) * weight + fractional / Self::SCALE,
            remainder: (fractional % Self::SCALE) as u64,
        });
    }

    fn merge(&mut self, other: Self) {
        let remainder = self.remainder + other.remainder;
        self.whole += other.whole + u128::from(remainder / u64::from(BoundedRatio::SCALE));
        self.remainder = remainder % u64::from(BoundedRatio::SCALE);
    }

    /// The sum over `divisor`, rounded to the nearest integer (halves up, as `CentsSats` rounds).
    fn div_round(self, divisor: u128) -> u128 {
        let (quotient, rest) = (self.whole / divisor, self.whole % divisor);
        // Up when the rest, `rest + remainder / SCALE`, is at least half the divisor. Both sides
        // stay far below u128::MAX: `divisor` is one BTC or a supply, in satoshis.
        let doubled_rest = 2 * (rest * Self::SCALE + u128::from(self.remainder));
        quotient + u128::from(doubled_rest >= divisor * Self::SCALE)
    }

    fn multiplied(self, price: u64) -> Option<Self> {
        let fractional = u128::from(self.remainder) * u128::from(price);
        Some(Self {
            whole: self
                .whole
                .checked_mul(u128::from(price))?
                .checked_add(fractional / Self::SCALE)?,
            remainder: (fractional % Self::SCALE) as u64,
        })
    }
}
