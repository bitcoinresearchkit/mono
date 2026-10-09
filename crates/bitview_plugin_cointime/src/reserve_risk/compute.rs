use bitview_compute::ComputeRollingMedianFromStarts;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::Float64;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Timestamp};
use vecdb::ReadableVec;

use super::{super::value, Vecs};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        mappings: &MappingsVecs,
        prices: &PriceVecs,
        value: &value::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.vocdd_median_1m.compute_rolling_median_from_starts(
            starting_height,
            &blocks.lookback._1m,
            &value.vocdd.sum._24h.height,
            exit,
        )?;

        // Monotonic timestamps: a header timestamp earlier than its parent's would
        // otherwise count the time between them twice.
        let timestamps = &mappings.timestamp.monotonic;
        let mut state = None;
        self.hodl_bank.compute_transform3(
            starting_height,
            &prices.spot.usd.height,
            &self.vocdd_median_1m,
            timestamps,
            |(height, price, median, timestamp, this): (_, Dollars, Float64, Timestamp, _)| {
                let (bank, previous) = state.get_or_insert_with(|| {
                    let previous = height.decremented();
                    (
                        previous
                            .and_then(|previous| this.collect_one(previous))
                            .map_or(0.0, f64::from),
                        previous
                            .and_then(|previous| timestamps.collect_one(previous))
                            .unwrap_or(timestamp),
                    )
                });
                let days = timestamp.difference_in_days_between_float(*previous);
                *previous = timestamp;
                *bank += (f64::from(price) - f64::from(median)) * days;
                (height, Float64::from(*bank))
            },
            exit,
        )?;

        Ok(())
    }
}
