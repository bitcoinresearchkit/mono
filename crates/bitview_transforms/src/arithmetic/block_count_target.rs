use bitview_primitives::StoredU64;
use brk_types::TARGET_BLOCKS_PER_DAY;
use vecdb::UnaryTransform;

/// Expected block count over a fixed number of days.
pub struct BlockCountTarget<const DAYS: u64>;

impl<I, const DAYS: u64> UnaryTransform<I, StoredU64> for BlockCountTarget<DAYS> {
    #[inline(always)]
    fn apply(_: I) -> StoredU64 {
        StoredU64::from(DAYS * TARGET_BLOCKS_PER_DAY)
    }
}
