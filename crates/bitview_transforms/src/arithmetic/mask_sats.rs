use bitview_primitives::Count;
use brk_types::Sats;
use vecdb::BinaryTransform;

pub struct MaskSats;

impl BinaryTransform<Count, Sats, Sats> for MaskSats {
    #[inline]
    fn apply(mask: Count, value: Sats) -> Sats {
        if u64::from(mask) != 0 {
            value
        } else {
            Sats::ZERO
        }
    }
}
