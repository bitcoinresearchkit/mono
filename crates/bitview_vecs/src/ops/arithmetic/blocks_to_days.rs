use bitview_primitives::{Count, Days};
use brk_types::TARGET_BLOCKS_PER_DAY_F32;
use vecdb::UnaryTransform;

pub struct BlocksToDays;

impl UnaryTransform<Count, Days> for BlocksToDays {
    #[inline(always)]
    fn apply(blocks: Count) -> Days {
        Days::new(*blocks as f32 / TARGET_BLOCKS_PER_DAY_F32)
    }
}
