use bitview_primitives::Bytes;
use bitview_types::{BlockSizeEntry, BlockSizesWeights, BlockWeightEntry, TimePeriod};
use brk_types::Weight;

use super::block_window::BlockWindow;
use crate::{Error, Query, Result};

impl Query {
    /// Time-bucketed average block size and weight over `time_period`. Returns
    /// two parallel vecs (one entry per bucket, ordered chronologically): byte
    /// size in `sizes`, weight units in `weights`. Each entry carries the
    /// bucket's average height/timestamp and the round-half-up mean of the
    /// corresponding metric. Single bucket-pass: built via `.map(...).unzip()`
    /// to avoid re-walking buckets.
    pub fn block_sizes_weights(&self, time_period: TimePeriod) -> Result<BlockSizesWeights> {
        let pin = self.pin_safe_lengths()?;
        let blocks = &self.indexer().vecs().blocks;
        let tip = pin.lengths().last_height().ok_or(Error::StateUpdating)?;
        let bw = BlockWindow::new_at(self, time_period, tip)?;

        let block_sizes: Vec<Bytes> = bw
            .read(&blocks.total)?
            .into_iter()
            .map(Bytes::from)
            .collect();
        let block_weights: Vec<Weight> = bw.read(&blocks.weight)?;
        drop(pin);

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
