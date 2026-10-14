use bitview_compute::ComputeRollingStats;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::LookbackVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::CachedSeries;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height};
use vecdb::ReadableVec;

use super::Vecs;

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        lookback: &LookbackVecs,
        prices: &PriceVecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.compute_start();
        let close = &prices.spot.cents.height;

        compute_sma_prefix(
            &mut self.sma_prefix_sum,
            starting_lengths.height,
            close,
            exit,
        )?;

        for (days, state) in self.ema_stored.iter_mut() {
            state.compute_rolling_ema(
                starting_lengths.height,
                lookback.start_vec(days),
                close,
                exit,
            )?;
        }

        let from = starting_lengths.height;
        self.sma.compute_ratios(from, close, exit)?;
        for (_, ema) in self.ema.iter_mut() {
            ema.compute_ratio(from, close, exit)?;
        }

        Ok(())
    }
}

fn compute_sma_prefix(
    target: &mut CachedSeries<Height, Cents>,
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
            (height, Cents::new(*sum))
        },
        exit,
    )?;
    Ok(())
}
