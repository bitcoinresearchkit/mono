use bitview_primitives::Count;
use brk_types::TARGET_BLOCKS_PER_DAY;
use vecdb::UnaryTransform;

/// Expected block count over a fixed number of days.
pub struct BlockCountTarget<const DAYS: u64>;

impl<I, const DAYS: u64> UnaryTransform<I, Count> for BlockCountTarget<DAYS> {
    #[inline(always)]
    fn apply(_: I) -> Count {
        Count::from(DAYS * TARGET_BLOCKS_PER_DAY)
    }
}
