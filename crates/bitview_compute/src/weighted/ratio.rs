use brk_types::BoundedRatio;

#[derive(Clone, Copy, Default)]
pub struct WeightedRatio {
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
    pub fn value(&self) -> BoundedRatio {
        if self.denominator > 0.0 {
            BoundedRatio::from((self.numerator / self.denominator).clamp(0.0, 1.0))
        } else {
            BoundedRatio::NAN
        }
    }
}
