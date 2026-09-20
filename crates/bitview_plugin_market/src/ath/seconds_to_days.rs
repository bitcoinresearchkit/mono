use brk_types::{ONE_DAY_IN_SEC_F64, StoredF32, StoredU32};
use vecdb::UnaryTransform;

pub(super) struct SecondsToDays;

impl UnaryTransform<StoredU32, StoredF32> for SecondsToDays {
    fn apply(seconds: StoredU32) -> StoredF32 {
        StoredF32::from(*seconds as f64 / ONE_DAY_IN_SEC_F64)
    }
}
