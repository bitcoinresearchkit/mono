use brk_types::{
    BlockSizeEntry, BlockSizesWeights, BlockWeightEntry, StoredU64, TimePeriod, Weight,
};

use super::block_window::BlockWindow;
use crate::Query;

impl Query {
    /// Time-bucketed average block size and weight over `time_period`. Returns
    /// two parallel vecs (one entry per bucket, ordered chronologically): byte
    /// size in `sizes`, weight units in `weights`. Each entry carries the
    /// bucket's average height/timestamp and the round-half-up mean of the
    /// corresponding metric. Single bucket-pass: built via `.map(...).unzip()`
    /// to avoid re-walking buckets.
    pub fn block_sizes_weights(
        &self,
        time_period: TimePeriod,
    ) -> brk_error::Result<BlockSizesWeights> {
        let blocks = &self.indexer().vecs().blocks;
        let bw = BlockWindow::new(self, time_period)?;

        let block_sizes: Vec<StoredU64> = bw.read(&blocks.total)?;
        let block_weights: Vec<Weight> = bw.read(&blocks.weight)?;

        let (sizes, weights) = bw
            .buckets
            .iter()
            .map(|b| {
                (
                    BlockSizeEntry {
                        avg_height: b.avg_height,
                        timestamp: b.avg_timestamp,
                        avg_size: u64::from(b.mean_rounded(&block_sizes)),
                    },
                    BlockWeightEntry {
                        avg_height: b.avg_height,
                        timestamp: b.avg_timestamp,
                        avg_weight: b.mean_rounded(&block_weights),
                    },
                )
            })
            .unzip();

        Ok(BlockSizesWeights { sizes, weights })
    }
}
