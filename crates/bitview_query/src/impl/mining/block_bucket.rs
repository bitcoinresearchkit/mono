use brk_types::{Height, Timestamp};

/// One time-bucket of blocks in a `BlockWindow`.
pub struct BlockBucket {
    pub avg_height: Height,
    pub avg_timestamp: Timestamp,
    pub(super) count: u32,
}
