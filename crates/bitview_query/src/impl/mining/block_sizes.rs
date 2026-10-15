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
        let bw = BlockWindow::new(self, time_period, tip)?;

        let sizes = bw.means_rounded(&*blocks.total, Bytes::from)?;
        let weights = bw.means_rounded(&*blocks.weight, Weight::from)?;
        drop(pin);

        let (sizes, weights) = bw
            .buckets
            .iter()
            .zip(sizes.into_iter().zip(weights))
            .map(|(b, (size, weight))| {
                (
                    BlockSizeEntry {
                        avg_height: b.avg_height,
                        timestamp: b.avg_timestamp,
                        avg_size: u64::from(size),
                    },
                    BlockWeightEntry {
                        avg_height: b.avg_height,
                        timestamp: b.avg_timestamp,
                        avg_weight: weight,
                    },
                )
            })
            .unzip();

        Ok(BlockSizesWeights { sizes, weights })
    }
}
