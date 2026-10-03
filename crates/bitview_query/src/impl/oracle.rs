use std::ops::Range;

use bitview_plugin_indexer::Lengths;
use bitview_plugin_price::{feed_blocks_for_warmup, feed_blocks_with};
use bitview_primitives::{Day1, TxOutIndex};
use brk_oracle::{
    Config, HistogramEma, HistogramEmaCompact, HistogramRaw, Oracle, cents_to_bin, sats_to_bin,
};
use brk_types::Dollars;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, Result};

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
        let _guard = self.read_publication()?;
        let safe = self.check_histogram_height(height)?;
        let seed = self.seed_bin_at(height)?;
        let _pin = self.pin_safe_lengths()?;
        drop(_guard);
        Ok(self
            .warm_oracle(seed, height + 1, &safe)?
            .ema()
            .to_compact())
    }

    /// Smoothed payment output histogram for a calendar `day`: the bin-by-bin average of
    /// every confirmed block's per-block EMA. The first block in each EMA config
    /// segment is reconstructed exactly, then later blocks in the segment are walked
    /// sequentially. Averaging keeps the result an intensive per-block rate rather
    /// than letting a busy day dominate.
    pub fn confirmed_payment_histogram_day(&self, day: Day1) -> Result<HistogramEmaCompact> {
        let _guard = self.read_publication()?;
        let safe = self.safe_lengths();
        let range = self.day_block_range(day, &safe)?;
        let segments = Config::segments_for_range(range)
            .map(|segment| self.seed_bin_at(segment.start).map(|seed| (segment, seed)))
            .collect::<Result<Vec<_>>>()?;
        let _pin = self.pin_safe_lengths()?;
        drop(_guard);
        Ok(self
            .average_payment_histogram_range(segments, &safe)?
            .to_compact())
    }

    fn average_payment_histogram_range(
        &self,
        segments: Vec<(Range<usize>, f64)>,
        safe: &Lengths,
    ) -> Result<HistogramEma> {
        let count: usize = segments.iter().map(|(range, _)| range.len()).sum();
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
        let _pin = self.pin_safe_lengths()?;
        Ok(match self.mempool() {
            Some(mempool) => mempool.live_raw_histogram(&self.tip_blockhash())?,
            None => HistogramRaw::zeros(),
        })
    }

    /// Unfiltered per-bin output counts for a confirmed `height`: every output
    /// in the block binned by value, with no payment filtering.
    pub fn confirmed_output_histogram(&self, height: usize) -> Result<HistogramRaw> {
        let _guard = self.read_publication()?;
        let safe = self.check_histogram_height(height)?;
        let _pin = self.pin_safe_lengths()?;
        drop(_guard);
        self.output_histogram_for_blocks(height..height + 1, &safe)
    }

    /// Unfiltered per-bin output counts for a calendar `day`: every block's output
    /// histogram summed bin-by-bin. Raw counts are additive, so the day total is
    /// just the sum across its confirmed blocks.
    pub fn confirmed_output_histogram_day(&self, day: Day1) -> Result<HistogramRaw> {
        let _guard = self.read_publication()?;
        let safe = self.safe_lengths();
        let range = self.day_block_range(day, &safe)?;
        let _pin = self.pin_safe_lengths()?;
        drop(_guard);
        self.output_histogram_for_blocks(range, &safe)
    }

    /// The live tip oracle: the committed base, with the published pool's
    /// mempool outputs blended in as a final slot when a mempool is configured.
    fn live_oracle(&self) -> Result<Oracle> {
        let _guard = self.read_publication()?;
        // Capture the completed mempool publication while the confirmed anchor
        // is held stable, before doing potentially expensive window work.
        let live = self
            .mempool()
            .map(|mempool| mempool.live_eligible_histogram(&self.tip_blockhash()))
            .transpose()?;
        let safe = self.safe_lengths();
        let height = safe.height.to_usize();
        let last = height
            .checked_sub(1)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_owned()))?;
        let seed_bin = self.seed_bin_at(last)?;
        let tip = self.tip_blockhash();
        let revision = self.indexer().publication().revision();
        let _pin = self.pin_safe_lengths()?;
        drop(_guard);
        let mut oracle = self
            .0
            .live_oracle
            .get_or_try_init(tip, revision, || self.warm_oracle(seed_bin, height, &safe))?;
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
        let start = end.saturating_sub(config.window_size);
        let mut warmed = Ok(());
        let oracle = Oracle::from_checkpoint(seed_bin, config, |o| {
            warmed = feed_blocks_for_warmup(o, self.indexer(), start..end, Some(safe));
        });
        warmed?;
        Ok(oracle)
    }

    /// Seed bin for an oracle warm-up: the stored spot price at `height` mapped
    /// `cents -> bin`. 404s when the oracle prices aren't computed that far yet,
    /// which also covers the stamp-before-write race where the vec length leads
    /// the readable data.
    fn seed_bin_at(&self, height: usize) -> Result<f64> {
        let cents = self
            .plugins()
            .price
            .spot
            .cents
            .height
            .collect_one_at(height)
            .ok_or_else(|| Error::NotFound("oracle prices not yet computed".to_string()))?;
        let cents = cents
            .finite_inner()
            .ok_or(Error::Internal("Invalid oracle seed price"))?;
        Ok(cents_to_bin(cents as f64))
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
    fn check_histogram_height(&self, height: usize) -> Result<Lengths> {
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
    fn day_block_range(&self, day: Day1, safe: &Lengths) -> Result<Range<usize>> {
        let first_height = &self.plugins().mappings.day1.first_height;
        let bound = self.histogram_bound(safe);
        let start = first_height
            .collect_one(day)
            .map_or(usize::MAX, |h| h.to_usize());
        let end = first_height
            .collect_one_at(usize::from(day) + 1)
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
