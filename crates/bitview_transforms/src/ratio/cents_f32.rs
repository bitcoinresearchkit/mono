use bitview_primitives::StoredF32;
use brk_types::Cents;
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
