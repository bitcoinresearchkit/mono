use bitview_primitives::OHLCCents;
use bitview_vecs::{AggFold, LazyAggVec};
use brk_types::{Cents, Height};
use rangeindex::RangeMap;
use vecdb::{ReadableVec, VecIndex};

/// Prices read at once for a group of adjacent candles.
const BATCH_PRICES: usize = 1 << 16;

/// OHLC candles derived directly from spot prices and period boundaries.
pub(super) type LazyOhlcVec<I> = LazyAggVec<I, OHLCCents, Height, Cents, Ohlc>;

/// Each period's candle from the spot prices between its boundaries; an empty period carries
/// the preceding close.
pub(super) struct Ohlc;

impl Ohlc {
    fn candle_from_prices(first: usize, end: usize, prices: &[Cents]) -> OHLCCents {
        if first >= end {
            let close = first
                .checked_sub(1)
                .and_then(|height| prices.get(height))
                .copied()
                .unwrap_or_default();
            return OHLCCents::from(close);
        }
        let mut candle = OHLCCents::from(prices[first]);
        for &price in &prices[first + 1..end] {
            candle.high = candle.high.max(price);
            candle.low = candle.low.min(price);
            candle.close = price;
        }
        candle
    }
}

impl AggFold<OHLCCents, Height, Cents> for Ohlc {
    fn try_fold<
        MI: VecIndex,
        S: ReadableVec<Height, Cents> + ?Sized,
        B,
        E,
        F: FnMut(B, OHLCCents) -> Result<B, E>,
    >(
        source: &S,
        mapping: &RangeMap<Height, MI>,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let first_heights = mapping.starts();
        let price_len = source.visible_len();
        let first = |index: usize| first_heights[index].to_usize().min(price_len);
        let end = |index: usize| {
            first_heights
                .get(index + 1)
                .map_or(price_len, |height| height.to_usize().min(price_len))
        };
        // Adjacent candles share one price read, so shared physical pages are decoded once,
        // in groups of about `BATCH_PRICES` (at least one candle) so the buffer stays bounded.
        // Each read includes the preceding close for a leading empty period.
        let mut acc = init;
        let mut prices = Vec::new();
        let mut index = from;
        while index < to {
            let price_from = first(index).saturating_sub(1);
            let mut group_end = index + 1;
            while group_end < to && end(group_end) - price_from <= BATCH_PRICES {
                group_end += 1;
            }
            prices.clear();
            source.read_into_at(price_from, end(group_end - 1), &mut prices);
            for index in index..group_end {
                acc = f(
                    acc,
                    Self::candle_from_prices(
                        first(index) - price_from,
                        end(index) - price_from,
                        &prices,
                    ),
                )?;
            }
            index = group_end;
        }
        Ok(acc)
    }

    fn collect_one<MI: VecIndex, S: ReadableVec<Height, Cents> + ?Sized>(
        source: &S,
        mapping: &RangeMap<Height, MI>,
        index: usize,
    ) -> Option<OHLCCents> {
        let boundaries = mapping.starts();
        let price_len = source.visible_len();
        let first = boundaries.get(index)?.to_usize().min(price_len);
        let end = boundaries
            .get(index + 1)
            .map_or(price_len, |height| height.to_usize().min(price_len));
        let from = first.saturating_sub(1);
        let prices = source.collect_range_dyn(from, end.max(first));
        Some(Self::candle_from_prices(
            first - from,
            end.saturating_sub(from),
            &prices,
        ))
    }

    /// Each run of adjacent requested candles shares one read; gaps are skipped.
    fn read_sorted_into<MI: VecIndex, S: ReadableVec<Height, Cents> + ?Sized>(
        source: &S,
        mapping: &RangeMap<Height, MI>,
        indices: &[usize],
        out: &mut Vec<OHLCCents>,
    ) {
        out.reserve(indices.len());
        for run in indices.chunk_by(|a, b| b == &(a + 1)) {
            Self::fold(
                source,
                mapping,
                run[0],
                run[run.len() - 1] + 1,
                (),
                |(), candle| out.push(candle),
            );
        }
    }
}
