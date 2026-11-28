use bitview_types::{BlockSizeEntry, BlockSizesWeights, BlockWeightEntry, TimePeriod};

use super::block_window::BlockWindow;
use crate::{Query, Result, TipPin};

impl Query {
    /// Round-half-up mean block sizes (bytes) and weights (WU) per time bucket over
    /// `time_period`, chronological, as two parallel vecs.
    pub fn block_sizes_weights(
        &self,
        at: &TipPin,
        time_period: TimePeriod,
    ) -> Result<BlockSizesWeights> {
        let blocks = &self.plugins().blocks;
        let tip = at.height;
        let bw = BlockWindow::new(self, time_period, tip)?;

        let sizes = bw.means_from_totals(&blocks.size.cumulative.height)?;
        let weights = bw.means_from_totals(&blocks.weight.cumulative.height)?;

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
