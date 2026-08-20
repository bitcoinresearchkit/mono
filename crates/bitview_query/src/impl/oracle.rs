use std::{ops::Range, sync::Arc};

use bitview_plugin_indexer::Lengths;
use bitview_plugin_price::Vecs as PricesVecs;
use brk_error::Error;
use brk_oracle::{
    Config, HistogramEma, HistogramEmaCompact, HistogramRaw, Oracle, cents_to_bin, sats_to_bin,
};
use brk_types::{Day1, Dollars, TxOutIndex};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::Query;

impl Query {
    pub fn live_price(&self) -> brk_error::Result<Dollars> {
        Ok(self.live_oracle()?.price_dollars())
    }

    /// Smoothed payment output histogram at the live tip, quantized for the wire.
    pub fn live_payment_histogram(&self) -> brk_error::Result<HistogramEmaCompact> {
        Ok(self.live_oracle()?.ema().to_compact())
    }

    /// Smoothed payment output histogram for a confirmed `height`, deterministically
    /// reconstructed by replaying the window ending at `height`. EMA values are
    /// seed-independent, so the result is exact.
    pub fn confirmed_payment_histogram(
        &self,
        height: usize,
    ) -> brk_error::Result<HistogramEmaCompact> {
        let safe = self.check_histogram_height(height)?;
        Ok(self.ema_oracle_at(height, &safe)?.ema().to_compact())
    }

    /// Smoothed payment output histogram for a calendar `day`: the bin-by-bin average of
    /// every confirmed block's per-block EMA. The first block in each EMA config
    /// segment is reconstructed exactly, then later blocks in the segment are walked
    /// sequentially. Averaging keeps the result an intensive per-block rate rather
    /// than letting a busy day dominate.
    pub fn confirmed_payment_histogram_day(
        &self,
        day: Day1,
    ) -> brk_error::Result<HistogramEmaCompact> {
        let safe = self.safe_lengths();
        let range = self.day_block_range(day, &safe)?;
        Ok(self
            .average_payment_histogram_range(range, &safe)?
            .to_compact())
    }

    fn average_payment_histogram_range(
        &self,
        range: Range<usize>,
        safe: &Lengths,
    ) -> brk_error::Result<HistogramEma> {
        let count = range.len();
        let mut acc = HistogramEma::zeros();

        for segment in Config::segments_for_range(range) {
            let mut oracle = self.ema_oracle_at(segment.start, safe)?;
            acc.add_from(oracle.ema());

            let feed_start = segment.start + 1;
            if feed_start < segment.end {
                PricesVecs::feed_blocks_with(
                    &mut oracle,
                    self.indexer(),
                    feed_start..segment.end,
                    Some(safe),
                    |_, oracle, _| acc.add_from(oracle.ema()),
                );
            }
        }

        acc.divide_by(count as f64);
        Ok(acc)
    }

    /// Unfiltered per-bin output counts at the live tip: every forming-block
    /// mempool output binned by value, with none of the round-dollar payment
    /// filters applied. Zeros when no mempool is configured.
    pub fn live_output_histogram(&self) -> brk_error::Result<HistogramRaw> {
        Ok(match self.mempool() {
            Some(mempool) => mempool.live_raw_histogram(),
            None => HistogramRaw::zeros(),
        })
    }

    /// Unfiltered per-bin output counts for a confirmed `height`: every output
    /// in the block binned by value, with no payment filtering.
    pub fn confirmed_output_histogram(&self, height: usize) -> brk_error::Result<HistogramRaw> {
        let safe = self.check_histogram_height(height)?;
        Ok(self.output_histogram_for_blocks(height..height + 1, &safe))
    }

    /// Unfiltered per-bin output counts for a calendar `day`: every block's output
    /// histogram summed bin-by-bin. Raw counts are additive, so the day total is
    /// just the sum across its confirmed blocks.
    pub fn confirmed_output_histogram_day(&self, day: Day1) -> brk_error::Result<HistogramRaw> {
        let safe = self.safe_lengths();
        let range = self.day_block_range(day, &safe)?;
        Ok(self.output_histogram_for_blocks(range, &safe))
    }

    /// The live tip oracle: the cached committed base, with the forming block's
    /// mempool outputs blended in as a final slot when a mempool is configured.
    fn live_oracle(&self) -> brk_error::Result<Oracle> {
        let mut oracle = (*self.cached_oracle()?).clone();
        if let Some(mempool) = self.mempool() {
            oracle.process_histogram(&mempool.live_eligible_histogram());
        }
        Ok(oracle)
    }

    /// Tip oracle warmed over the last `window_size` committed blocks, seeded
    /// from the last committed price. Cached per tip height; rebuilt on advance
    /// or reorg.
    fn cached_oracle(&self) -> brk_error::Result<Arc<Oracle>> {
        let safe = self.safe_lengths();
        let height = safe.height;

        if let Some(oracle) = self
            .0
            .live_oracle
            .read()
            .unwrap()
            .as_ref()
            .filter(|(h, _)| *h == height)
            .map(|(_, o)| o.clone())
        {
            return Ok(oracle);
        }

        let last = self
            .plugins()
            .price
            .spot
            .cents
            .height
            .len()
            .saturating_sub(1);
        let seed_bin = self.seed_bin_at(last)?;
        let oracle = Arc::new(self.warm_oracle(seed_bin, height.to_usize(), &safe));

        let mut cache = self.0.live_oracle.write().unwrap();
        if cache.as_ref().is_none_or(|(h, _)| *h != height) {
            *cache = Some((height, oracle.clone()));
        }
        Ok(oracle)
    }

    /// Oracle warmed to just after `height`, ready for its per-block EMA. Seeds
    /// from the stored spot price at `height`, though the EMA is seed-independent
    /// so the seed only sets the price read-out, not the window contents.
    fn ema_oracle_at(&self, height: usize, safe: &Lengths) -> brk_error::Result<Oracle> {
        let seed_bin = self.seed_bin_at(height)?;
        Ok(self.warm_oracle(seed_bin, height + 1, safe))
    }

    /// An oracle seeded at `seed_bin` and warmed by replaying the `window_size`
    /// committed blocks ending just before `end`. Reads are capped at `safe` so
    /// concurrent indexer writes past the cap stay invisible.
    fn warm_oracle(&self, seed_bin: f64, end: usize, safe: &Lengths) -> Oracle {
        let config = Config::for_height(end.saturating_sub(1));
        let start = end.saturating_sub(config.window_size);
        Oracle::from_checkpoint(seed_bin, config, |o| {
            PricesVecs::feed_blocks_for_warmup(o, self.indexer(), start..end, Some(safe));
        })
    }

    /// Seed bin for an oracle warm-up: the stored spot price at `height` mapped
    /// `cents -> bin`. 404s when the oracle prices aren't computed that far yet,
    /// which also covers the stamp-before-write race where the vec length leads
    /// the readable data.
    fn seed_bin_at(&self, height: usize) -> brk_error::Result<f64> {
        let cents = self
            .plugins()
            .price
            .spot
            .cents
            .height
            .collect_one_at(height)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_string()))?;
        Ok(cents_to_bin(cents.inner() as f64))
    }

    fn histogram_bound(&self, safe: &Lengths) -> usize {
        self.plugins()
            .price
            .spot
            .cents
            .height
            .len()
            .min(safe.height.to_usize())
    }

    /// `height < min(spot price len, safe height)` or 404.
    /// Returns the safe lengths so callers cap reads at the same bound.
    fn check_histogram_height(&self, height: usize) -> brk_error::Result<Lengths> {
        let safe = self.safe_lengths();
        let bound = self.histogram_bound(&safe);
        if height >= bound {
            return Err(Error::NotFound(format!(
                "oracle histogram unavailable for height {height}"
            )));
        }
        Ok(safe)
    }

    /// The confirmed block heights `[first, end)` of calendar `day`, clamped to
    /// the same histogram-available bound as `check_histogram_height`. 404 when
    /// the day has no committed blocks in range.
    fn day_block_range(&self, day: Day1, safe: &Lengths) -> brk_error::Result<Range<usize>> {
        let first_height = &self.plugins().mappings.day1.first_height;
        let bound = self.histogram_bound(safe);
        let start = first_height
            .collect_one(day)
            .map_or(usize::MAX, |h| h.to_usize());
        let end = first_height
            .collect_one(day + 1)
            .map_or(bound, |h| h.to_usize())
            .min(bound);
        if start >= end {
            return Err(Error::NotFound(format!(
                "oracle histogram unavailable for day {day}"
            )));
        }
        Ok(start..end)
    }

    /// Unfiltered histogram for a contiguous confirmed block range: every output,
    /// coinbase included, binned by value via `sats_to_bin` with no payment
    /// filtering. Raw counts are additive, so a day can be read as one output
    /// range instead of one block at a time.
    fn output_histogram_for_blocks(&self, range: Range<usize>, safe: &Lengths) -> HistogramRaw {
        let indexer = self.indexer();
        let safe_height = safe.height.to_usize();
        let total_outputs = safe.txout_index.to_usize();
        let first_txout_index = &indexer.vecs().outputs.first_txout_index;
        let out_start = first_txout_index
            .collect_one_at(range.start)
            .unwrap()
            .to_usize();
        let out_end = if range.end < safe_height {
            first_txout_index.collect_one_at(range.end).unwrap()
        } else {
            TxOutIndex::from(total_outputs)
        }
        .to_usize();

        let mut hist = HistogramRaw::zeros();
        indexer
            .vecs()
            .outputs
            .value
            .for_each_range_at(out_start, out_end, |sats| {
                if let Some(bin) = sats_to_bin(sats) {
                    hist.increment(bin);
                }
            });
        hist
    }
}
