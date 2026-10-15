use std::ops::Range;

use bitview_plugin_indexer::SafeLengths;
use bitview_plugin_price::{feed_blocks_for_warmup, feed_blocks_with};
use bitview_primitives::Lengths;
use bitview_primitives::{Day1, TxOutIndex};
use brk_oracle::{
    Config, HistogramEma, HistogramEmaCompact, HistogramRaw, Oracle, START_HEIGHT_SLOW, sats_to_bin,
};
use brk_types::Dollars;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, Result, check_bound};

impl Query {
    pub fn live_price(&self) -> Result<Dollars> {
        Ok(self.live_oracle()?.price_dollars())
    }

    /// Smoothed payment output histogram at the live tip, quantized for the wire.
    pub fn live_payment_histogram(&self) -> Result<HistogramEmaCompact> {
        Ok(self.live_oracle()?.ema().to_compact())
    }

    /// Smoothed payment output histogram for a confirmed `height`, deterministically
    /// reconstructed by replaying the window ending at `height`. EMA values are
    /// seed-independent, so the result is exact.
    pub fn confirmed_payment_histogram(&self, height: usize) -> Result<HistogramEmaCompact> {
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, height, pin.lengths().height.to_usize())?;
        let safe = self.check_histogram_height(height, pin.lengths())?;
        let seed = self.seed_bin_at(height)?;
        Ok(self
            .warm_oracle(seed, height + 1, &safe)?
            .ema()
            .to_compact())
    }

    /// Smoothed payment output histogram for a calendar `day`: the bin-by-bin average of
    /// every confirmed block's per-block EMA. The first block in each EMA config
    /// segment is reconstructed exactly, then later blocks in the segment are walked
    /// sequentially. Blocks before the oracle's first one count as zeros, as their height
    /// reads do. Averaging keeps the result an intensive per-block rate rather than letting
    /// a busy day dominate.
    pub fn confirmed_payment_histogram_day(&self, day: Day1) -> Result<HistogramEmaCompact> {
        let pin = self.pin_safe_lengths()?;
        let safe = pin.lengths();
        let range = self.day_block_range(day, &pin)?;
        let count = range.len();
        let fed = range.start.max(START_HEIGHT_SLOW)..range.end;
        let segments = Config::segments_for_range(fed)
            .map(|segment| self.seed_bin_at(segment.start).map(|seed| (segment, seed)))
            .collect::<Result<Vec<_>>>()?;
        Ok(self
            .average_payment_histogram_range(segments, count, &safe)?
            .to_compact())
    }

    /// The segments' per-block EMAs averaged bin by bin over `count` blocks.
    fn average_payment_histogram_range(
        &self,
        segments: Vec<(Range<usize>, f64)>,
        count: usize,
        safe: &Lengths,
    ) -> Result<HistogramEma> {
        let mut acc = HistogramEma::zeros();

        for (segment, seed) in segments {
            let mut oracle = self.warm_oracle(seed, segment.start + 1, safe)?;
            acc.add_from(oracle.ema());

            let feed_start = segment.start + 1;
            if feed_start < segment.end {
                feed_blocks_with(
                    &mut oracle,
                    self.indexer(),
                    feed_start..segment.end,
                    Some(safe),
                    |_, oracle, _| acc.add_from(oracle.ema()),
                )?;
            }
        }

        acc.divide_by(count as f64);
        Ok(acc)
    }

    /// Unfiltered per-bin output counts at the live tip: every published
    /// mempool output binned by value, with none of the round-dollar payment
    /// filters applied. Zeros when no mempool is configured.
    pub fn live_output_histogram(&self) -> Result<HistogramRaw> {
        Ok(match Self::lookup_mempool(self.mempool()) {
            Some((mempool, tip)) => mempool.live_raw_histogram(&tip)?,
            None => HistogramRaw::zeros(),
        })
    }

    /// Unfiltered per-bin output counts for a confirmed `height`: every output
    /// in the block binned by value, with no payment filtering.
    pub fn confirmed_output_histogram(&self, height: usize) -> Result<HistogramRaw> {
        let pin = self.pin_safe_lengths()?;
        check_bound(&pin, height, pin.lengths().height.to_usize())?;
        let safe = self.check_histogram_height(height, pin.lengths())?;
        self.output_histogram_for_blocks(height..height + 1, &safe)
    }

    /// Unfiltered per-bin output counts for a calendar `day`: every block's output
    /// histogram summed bin-by-bin. Raw counts are additive, so the day total is
    /// just the sum across its confirmed blocks.
    pub fn confirmed_output_histogram_day(&self, day: Day1) -> Result<HistogramRaw> {
        let pin = self.pin_safe_lengths()?;
        let range = self.day_block_range(day, &pin)?;
        self.output_histogram_for_blocks(range, &pin.lengths())
    }

    /// The live tip oracle: the committed base, with the published pool's
    /// mempool outputs blended in as a final slot when a mempool is configured.
    fn live_oracle(&self) -> Result<Oracle> {
        let (mempool, pin) = self.mempool_then_pin()?;
        let tip = pin.tip();
        // Capture the completed mempool publication while the confirmed anchor
        // is held stable, before doing potentially expensive window work. A pool
        // not yet at this tip has nothing to blend.
        let live = match mempool.map(|mempool| mempool.live_eligible_histogram(&tip)) {
            Some(Ok(histogram)) => Some(histogram),
            None | Some(Err(brk_error::Error::StateUpdating)) => None,
            Some(Err(error)) => return Err(error.into()),
        };
        let safe = pin.lengths();
        let height = safe.height.to_usize();
        let last = height
            .checked_sub(1)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_owned()))?;
        let seed_bin = self.seed_bin_at(last)?;
        let mut oracle = self
            .0
            .live_oracle
            .get_or_try_init(pin.revision(), || self.warm_oracle(seed_bin, height, &safe))?;
        if let Some(histogram) = live {
            oracle.process_histogram(&histogram);
        }
        Ok(oracle)
    }

    /// An oracle seeded at `seed_bin` and warmed by replaying the `window_size`
    /// committed blocks ending just before `end`. Reads are capped at `safe` so
    /// concurrent indexer writes past the cap stay invisible.
    fn warm_oracle(&self, seed_bin: f64, end: usize, safe: &Lengths) -> Result<Oracle> {
        let config = Config::for_height(end.saturating_sub(1));
        // Like the price plugin: the oracle's window never reaches below its first block.
        let start = end
            .saturating_sub(config.window_size)
            .max(START_HEIGHT_SLOW.min(end));
        Ok(Oracle::from_checkpoint(seed_bin, config, |o| {
            feed_blocks_for_warmup(o, self.indexer(), start..end, Some(safe))
        })?)
    }

    /// Seed bin for an oracle warm-up: the oracle's exact reference bin at `height`.
    /// 404s when the oracle prices aren't computed that far yet, which also covers
    /// the stamp-before-write race where the vec length leads the readable data.
    fn seed_bin_at(&self, height: usize) -> Result<f64> {
        self.plugins()
            .price
            .oracle_bin
            .collect_one_at(height)
            .map(f64::from)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_string()))
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
    fn check_histogram_height(&self, height: usize, safe: Lengths) -> Result<Lengths> {
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
    /// the day has no committed blocks in range; 503 while a lowered publication hides them.
    fn day_block_range(&self, day: Day1, pin: &SafeLengths) -> Result<Range<usize>> {
        let first_height = &self.plugins().mappings.day1.first_height;
        let bound = self.histogram_bound(&pin.lengths());
        let start = first_height
            .collect_one(day)
            .map_or(usize::MAX, |h| h.to_usize());
        let end = first_height
            .collect_one_at(usize::from(day) + 1)
            .map_or(bound, |h| h.to_usize())
            .min(bound);
        if start >= end {
            check_bound(pin, start, bound)?;
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
    fn output_histogram_for_blocks(
        &self,
        range: Range<usize>,
        safe: &Lengths,
    ) -> Result<HistogramRaw> {
        let indexer = self.indexer();
        let safe_height = safe.height.to_usize();
        let total_outputs = safe.txout_index.to_usize();
        if range.start >= range.end || range.end > safe_height {
            return Err(Error::Internal("Invalid oracle block range"));
        }
        let first_txout_index = &indexer.vecs().outputs.first_txout_index;
        let out_start = first_txout_index
            .collect_one_at(range.start)
            .data()?
            .to_usize();
        let out_end = if range.end < safe_height {
            first_txout_index.collect_one_at(range.end).data()?
        } else {
            TxOutIndex::from(total_outputs)
        }
        .to_usize();
        if out_start > out_end || out_end > total_outputs {
            return Err(Error::Internal("Invalid oracle output boundaries"));
        }

        let mut hist = HistogramRaw::zeros();
        indexer
            .vecs()
            .outputs
            .value
            .try_fold_range_at(out_start, out_end, (), |(), sats| {
                if let Some(bin) = sats_to_bin(sats) {
                    let count = hist
                        .get_mut(bin)
                        .ok_or(Error::Internal("Invalid oracle bin"))?;
                    *count = count
                        .checked_add(1)
                        .ok_or(Error::Internal("Oracle histogram count overflow"))?;
                }
                Ok::<_, Error>(())
            })?;
        Ok(hist)
    }
}
