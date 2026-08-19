use brk_types::{Cents, StoredF32};
use vecdb::{BinaryTransform, unlikely};

pub struct RatioCentsF32;

impl BinaryTransform<Cents, Cents, StoredF32> for RatioCentsF32 {
    #[inline(always)]
    fn apply(numerator: Cents, denominator: Cents) -> StoredF32 {
        let denominator = f64::from(denominator);
        if unlikely(denominator == 0.0) {
            StoredF32::from(1.0)
        } else {
            StoredF32::from(f64::from(numerator) / denominator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagates_nan() {
        assert!(RatioCentsF32::apply(Cents::NAN, Cents::new(100)).is_nan());
        assert!(RatioCentsF32::apply(Cents::new(100), Cents::NAN).is_nan());
    }
}
