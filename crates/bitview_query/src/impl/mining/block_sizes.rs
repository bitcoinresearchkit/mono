use bitview_primitives::Bytes;
use bitview_types::{BlockSizeEntry, BlockSizesWeights, BlockWeightEntry, TimePeriod};
use brk_types::Weight;

use super::block_window::BlockWindow;
use crate::{Query, Result};

impl Query {
    /// Round-half-up mean block sizes (bytes) and weights (WU) per time bucket over
    /// `time_period`, chronological, as two parallel vecs.
    pub fn block_sizes_weights(&self, time_period: TimePeriod) -> Result<BlockSizesWeights> {
        let pin = self.pin_safe_lengths()?;
        let blocks = &self.indexer().vecs().blocks;
        let tip = Query::tip_height(&pin)?;
        let bw = BlockWindow::new(self, time_period, tip)?;

        let sizes = bw.means_rounded(&*blocks.size, Bytes::from)?;
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
