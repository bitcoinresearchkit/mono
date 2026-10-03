use bitview_primitives::{StoredU32, StoredU64};
use brk_types::Sats;
use vecdb::BinaryTransform;

pub struct MaskSats;

impl BinaryTransform<StoredU32, Sats, Sats> for MaskSats {
    #[inline(always)]
    fn apply(mask: StoredU32, value: Sats) -> Sats {
        if mask == StoredU32::ONE {
            value
        } else {
            Sats::ZERO
        }
    }
}

impl BinaryTransform<StoredU64, Sats, Sats> for MaskSats {
    #[inline]
    fn apply(mask: StoredU64, value: Sats) -> Sats {
        if u64::from(mask) != 0 {
            value
        } else {
            Sats::ZERO
        }
    }
}
