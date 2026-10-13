use bitview_primitives::BoundedRatio;

/// A weighted ratio of disjoint contributions; weights are defined (see `defined_weight`).
#[derive(Clone, Copy, Default)]
pub struct WeightedRatio {
    numerator: f64,
    denominator: f64,
}

impl WeightedRatio {
    #[inline]
    pub(crate) fn add(&mut self, numerator: f64, denominator: f64, weight: f64) {
        debug_assert!(!weight.is_nan(), "undefined weight");
        if denominator == 0.0 || weight == 0.0 {
            return;
        }
        self.numerator += numerator * weight;
        self.denominator += denominator * weight;
    }

    #[inline]
    pub(crate) fn merge(&mut self, other: Self) {
        self.numerator += other.numerator;
        self.denominator += other.denominator;
    }

    #[inline]
    pub fn value(&self) -> BoundedRatio {
        if self.denominator > 0.0 {
            BoundedRatio::from((self.numerator / self.denominator).clamp(0.0, 1.0))
        } else {
            BoundedRatio::NAN
        }
    }
}
