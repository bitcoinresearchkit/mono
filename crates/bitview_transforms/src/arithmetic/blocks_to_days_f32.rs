use bitview_primitives::{Count, StoredF32};
use brk_types::TARGET_BLOCKS_PER_DAY_F32;
use vecdb::UnaryTransform;

pub struct BlocksToDaysF32;

impl UnaryTransform<Count, StoredF32> for BlocksToDaysF32 {
    #[inline(always)]
    fn apply(blocks: Count) -> StoredF32 {
        (*blocks as f32 / TARGET_BLOCKS_PER_DAY_F32).into()
    }
}
