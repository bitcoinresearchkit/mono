use bitview_primitives::{StoredU32, StoredU64};
use vecdb::UnaryTransform;

pub struct StoredU64ToStoredU32;

impl UnaryTransform<StoredU64, StoredU32> for StoredU64ToStoredU32 {
    #[inline(always)]
    fn apply(value: StoredU64) -> StoredU32 {
        let value = u64::from(value);
        debug_assert!(u32::try_from(value).is_ok());
        StoredU32::new(value as u32)
    }
}
