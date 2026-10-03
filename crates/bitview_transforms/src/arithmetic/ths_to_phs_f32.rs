use bitview_primitives::StoredF32;
use vecdb::UnaryTransform;

pub struct ThsToPhsF32;

impl UnaryTransform<StoredF32, StoredF32> for ThsToPhsF32 {
    #[inline(always)]
    fn apply(ths: StoredF32) -> StoredF32 {
        (*ths * 1000.0).into()
    }
}
