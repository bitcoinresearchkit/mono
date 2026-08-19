use brk_types::StoredF64;

#[derive(Clone, Copy, Default)]
pub struct WeightedRatio {
    numerator: f64,
    denominator: f64,
}

impl WeightedRatio {
    #[inline]
    pub fn add(&mut self, numerator: f64, denominator: f64, weight: f64) {
        if weight.is_finite() && weight > 0.0 {
            self.numerator += numerator * weight;
            self.denominator += denominator * weight;
        }
    }

    #[inline]
    pub fn merge(&mut self, other: Self) {
        self.numerator += other.numerator;
        self.denominator += other.denominator;
    }

    #[inline]
    pub fn value(&self) -> StoredF64 {
        if self.denominator > 0.0 {
            StoredF64::from((self.numerator / self.denominator).clamp(0.0, 1.0))
        } else {
            StoredF64::NAN
        }
    }
}
