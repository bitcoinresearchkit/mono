use brk_types::{Sats, StoredU64};
use vecdb::UnaryTransform;

pub struct StoredU64ToSats;

impl UnaryTransform<StoredU64, Sats> for StoredU64ToSats {
    #[inline(always)]
    fn apply(value: StoredU64) -> Sats {
        Sats::new(value.into())
    }
}
