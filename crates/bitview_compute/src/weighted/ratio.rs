use bitview_primitives::BoundedRatio;

/// A populated contribution with an undefined weight makes the whole ratio undefined.
#[derive(Clone, Copy, Default)]
pub struct WeightedRatio {
    numerator: f64,
    denominator: f64,
    undefined: bool,
}

impl WeightedRatio {
    #[inline]
    pub(crate) fn add(&mut self, numerator: f64, denominator: f64, weight: f64) {
        if denominator == 0.0 || weight == 0.0 {
            return;
        }
        if weight.is_nan() {
            self.undefined = true;
            return;
        }
        self.numerator += numerator * weight;
        self.denominator += denominator * weight;
    }

    #[inline]
    pub(crate) fn merge(&mut self, other: Self) {
        self.numerator += other.numerator;
        self.denominator += other.denominator;
        self.undefined |= other.undefined;
    }

    #[inline]
    pub fn value(&self) -> BoundedRatio {
        if !self.undefined && self.denominator > 0.0 {
            BoundedRatio::from((self.numerator / self.denominator).clamp(0.0, 1.0))
        } else {
            BoundedRatio::NAN
        }
    }
}
