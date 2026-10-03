use bitview_compute::ComputeRollingStats;
use bitview_plugin_blocks::Vecs as BlocksVecs;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, StoredU64};
use vecdb::ReadableVec;

use super::{Vecs, vecs::EmaPeriodId};

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        blocks: &BlocksVecs,
        prices: &PriceVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();
        let close = &prices.spot.cents.height;

        compute_sma_prefix(
            &mut self.sma_prefix_sum,
            starting_lengths.height,
            close,
            exit,
        )?;

        for &period in EmaPeriodId::ALL {
            period
                .select_mut(&mut self.ema_stored)
                .compute_rolling_ema(
                    starting_lengths.height,
                    blocks.lookback.start_vec(period.days()),
                    close,
                    exit,
                )?;
        }

        Ok(())
    }
}

fn compute_sma_prefix(
    target: &mut CachedSeries<Height, StoredU64>,
    from: Height,
    prices: &impl ReadableVec<Height, Cents>,
    exit: &Exit,
) -> Result<()> {
    let mut sum = None;
    target.compute_transform(
        from,
        prices,
        |(height, price, target)| {
            let sum = sum.get_or_insert_with(|| {
                height
                    .decremented()
                    .and_then(|height| target.collect_one(height))
                    .map(u64::from)
                    .unwrap_or_default()
            });
            *sum = sum
                .checked_add(price.inner())
                .expect("price SMA prefix sum overflow");
            (height, StoredU64::from(*sum))
        },
        exit,
    )?;
    Ok(())
}
