use bitview_primitives::StoredF32;
use brk_types::Cents;
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
