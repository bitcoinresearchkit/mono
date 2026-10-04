use bitview_primitives::{Seconds, StoredF32};
use brk_types::ONE_DAY_IN_SEC_F64;
use vecdb::UnaryTransform;

pub(super) struct SecondsToDays;

impl UnaryTransform<Seconds, StoredF32> for SecondsToDays {
    fn apply(seconds: Seconds) -> StoredF32 {
        StoredF32::from(*seconds as f64 / ONE_DAY_IN_SEC_F64)
    }
}
