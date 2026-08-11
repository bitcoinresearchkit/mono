use brk_types::{Cents, StoredU64};
use vecdb::UnaryTransform;

pub struct StoredU64ToCents;

impl UnaryTransform<StoredU64, Cents> for StoredU64ToCents {
    #[inline(always)]
    fn apply(value: StoredU64) -> Cents {
        Cents::new(value.into())
    }
}
