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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulation_stays_full_precision_until_encoding() {
        let mut ratio = WeightedRatio::default();
        assert!(ratio.value().is_nan());
        ratio.add(1.0, 3.0, 0.123456789123);
        let mut other = WeightedRatio::default();
        other.add(2.0, 5.0, 0.987654321987);
        ratio.merge(other);
        let exact =
            (0.123456789123 + 2.0 * 0.987654321987) / (3.0 * 0.123456789123 + 5.0 * 0.987654321987);
        assert_eq!(ratio.value(), BoundedRatio::from(exact));
        let before = ratio.value();
        ratio.add(1.0, 1.0, f64::NAN);
        ratio.add(1.0, 1.0, 0.0);
        assert_eq!(ratio.value(), before);
    }
}
