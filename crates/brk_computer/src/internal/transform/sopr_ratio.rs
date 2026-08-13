use brk_types::{Cents, StoredF32};
use vecdb::{BinaryTransform, unlikely};

pub struct SoprRatio;

impl BinaryTransform<Cents, Cents, StoredF32> for SoprRatio {
    #[inline(always)]
    fn apply(value_created: Cents, value_destroyed: Cents) -> StoredF32 {
        let value_destroyed = f64::from(value_destroyed);
        if unlikely(value_destroyed == 0.0) {
            StoredF32::from(1.0)
        } else {
            StoredF32::from(f64::from(value_created) / value_destroyed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_destroyed_value_is_one() {
        assert_eq!(*SoprRatio::apply(Cents::new(100), Cents::ZERO), 1.0);
    }

    #[test]
    fn nan_values_propagate() {
        assert!(SoprRatio::apply(Cents::NAN, Cents::new(100)).is_nan());
        assert!(SoprRatio::apply(Cents::new(100), Cents::NAN).is_nan());
    }
}
