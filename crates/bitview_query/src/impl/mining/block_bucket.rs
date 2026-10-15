use brk_types::{Height, Timestamp};

/// One time-bucket of blocks in a `BlockWindow`.
pub struct BlockBucket {
    pub avg_height: Height,
    pub avg_timestamp: Timestamp,
    /// Blocks in the bucket.
    pub(super) count: u32,
}
