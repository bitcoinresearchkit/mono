use bitview_primitives::{StoredU64, Weight64};
use brk_types::VSize;
use vecdb::UnaryTransform;

pub struct VBytesToWeight;

impl UnaryTransform<StoredU64, Weight64> for VBytesToWeight {
    #[inline(always)]
    fn apply(vbytes: StoredU64) -> Weight64 {
        Weight64::from(VSize::new(*vbytes))
    }
}
