use brk_types::{BoundedRatio, Cents, CentsSats, CentsSquaredSats};

/// Capitalized price using the same encoded weight for both raw cost-basis
/// moments. Fractional raw units are retained; only the final cents are floored.
/// Inputs are disjoint cohorts whose unweighted moment totals fit in u128.
#[derive(Clone, Copy, Default)]
pub struct WeightedCapitalizedPrice {
    numerator: WeightedSum,
    denominator: WeightedSum,
    undefined: bool,
}

impl WeightedCapitalizedPrice {
    pub(crate) fn add(
        &mut self,
        cap: CentsSats,
        capitalized_cap: CentsSquaredSats,
        weight: BoundedRatio,
    ) {
        if cap == CentsSats::ZERO || weight == BoundedRatio::ZERO {
            return;
        }
        if weight.is_nan() {
            self.undefined = true;
            return;
        }
        self.numerator.add(capitalized_cap.inner(), weight);
        self.denominator.add(cap.inner(), weight);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.numerator.merge(other.numerator);
        self.denominator.merge(other.denominator);
        self.undefined |= other.undefined;
    }

    pub fn value(&self) -> Cents {
        if self.undefined {
            return Cents::NAN;
        }
        if self.denominator == WeightedSum::default() {
            return Cents::ZERO;
        }
        // Find floor(numerator / denominator) without overflowing either raw
        // moment by multiplying it by the weight scale or the candidate price.
        let mut low = 0;
        let mut high = self
            .numerator
            .whole
            .checked_div(self.denominator.whole)
            .unwrap_or(Cents::MAX_FINITE.as_u128())
            .min(Cents::MAX_FINITE.as_u128()) as u64;
        // With substantial invested value the integer-parts quotient is
        // usually already exact, so avoid searching in the common case.
        if self
            .denominator
            .multiplied(high)
            .is_some_and(|value| value <= self.numerator)
        {
            return Cents::new(high);
        }
        while low < high {
            let mid = low + (high - low) / 2 + (high - low) % 2;
            if self
                .denominator
                .multiplied(mid)
                .is_some_and(|value| value <= self.numerator)
            {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        Cents::new(low)
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
