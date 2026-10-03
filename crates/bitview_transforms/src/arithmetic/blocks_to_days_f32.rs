use bitview_primitives::{StoredF32, StoredU32};
use brk_types::TARGET_BLOCKS_PER_DAY_F32;
use vecdb::UnaryTransform;

pub struct BlocksToDaysF32;

impl UnaryTransform<StoredU32, StoredF32> for BlocksToDaysF32 {
    #[inline(always)]
    fn apply(blocks: StoredU32) -> StoredF32 {
        (*blocks as f32 / TARGET_BLOCKS_PER_DAY_F32).into()
    }
}
