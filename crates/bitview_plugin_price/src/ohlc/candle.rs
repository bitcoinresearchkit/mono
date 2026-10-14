use std::{convert::Infallible, iter, sync::Arc};

use bitview_primitives::{Close, High, Low, OHLCCents};
use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Cents, Height, Version};
use rangeindex::SharedRangeMap;
use vecdb::{
    AnyExportableVec, AnyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec,
    VecIndex, short_type_name,
};

/// Prices read at once for a group of adjacent candles.
const BATCH_PRICES: usize = 1 << 16;

/// OHLC candles derived directly from spot prices and period boundaries.
#[derive(Clone)]
pub(super) struct LazyOhlcVec<I: VecIndex> {
    name: Arc<str>,
    base_version: Version,
    prices: ReadableBoxedVec<Height, Cents>,
    first_heights: SharedRangeMap<Height, I>,
}

impl<I: VecIndex> LazyOhlcVec<I> {
    /// `version` includes the boundary mapping's schema version.
    pub(super) fn new(
        name: &str,
        version: Version,
        prices: &(impl ReadableCloneableVec<Height, Cents> + ?Sized),
        first_heights: SharedRangeMap<Height, I>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            prices: prices.read_only_boxed_clone(),
            first_heights,
        }
    }

    fn try_for_each_candle<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(OHLCCents) -> Result<(), E>,
    ) -> Result<(), E> {
        let mapping = self.first_heights.load();
        let to = to.min(mapping.len());
        if from >= to {
            return Ok(());
        }
        let first_heights = mapping.starts();
        let price_len = self.prices.visible_len();
        let first = |index: usize| first_heights[from + index].to_usize().min(price_len);
        let end = |index: usize| {
            first_heights
                .get(from + index + 1)
                .map_or(price_len, |height| height.to_usize().min(price_len))
        };
        // Adjacent candles share one price read, so shared physical pages are decoded once,
        // in groups of about `BATCH_PRICES` (at least one candle) so the buffer stays bounded.
        // Each read includes the preceding close for a leading empty period.
        let count = to - from;
        let mut prices = Vec::new();
        let mut index = 0;
        while index < count {
            let price_from = first(index).saturating_sub(1);
            let mut group_end = index + 1;
            while group_end < count && end(group_end) - price_from <= BATCH_PRICES {
                group_end += 1;
            }
            prices.clear();
            self.prices
                .read_into_at(price_from, end(group_end - 1), &mut prices);
            for index in index..group_end {
                each(Self::candle_from_prices(
                    first(index) - price_from,
                    end(index) - price_from,
                    &prices,
                ))?;
            }
            index = group_end;
        }

        Ok(())
    }

    fn for_each_candle(&self, from: usize, to: usize, mut each: impl FnMut(OHLCCents)) {
        let result = self.try_for_each_candle(from, to, |candle| {
            each(candle);
            Ok::<_, Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }

    fn candle(&self, first: usize, end: usize) -> OHLCCents {
        let from = first.saturating_sub(1);
        let prices = self.prices.collect_range_dyn(from, end.max(first));
        Self::candle_from_prices(first - from, end.saturating_sub(from), &prices)
    }

    fn candle_from_prices(first: usize, end: usize, prices: &[Cents]) -> OHLCCents {
        if first >= end {
            let close = first
                .checked_sub(1)
                .and_then(|height| prices.get(height))
                .copied()
                .unwrap_or_default();
            return OHLCCents::from(Close::new(close));
        }
        let mut candle = OHLCCents::from(Close::new(prices[first]));
        for &price in &prices[first + 1..end] {
            candle.high = candle.high.max(High::new(price));
            candle.low = candle.low.min(Low::new(price));
            candle.close = Close::new(price);
        }
        candle
    }
}

impl<I: VecIndex> AnyVec for LazyOhlcVec<I> {
    fn version(&self) -> Version {
        self.base_version + self.prices.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.first_heights.load().len()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<OHLCCents>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<OHLCCents>()
    }
}

impl<I: VecIndex> TypedVec for LazyOhlcVec<I> {
    type I = I;
    type T = OHLCCents;
}

impl<I: VecIndex> ReadableVec<I, OHLCCents> for LazyOhlcVec<I> {
    vecdb::visitor_read_methods!(OHLCCents, for_each_candle, try_for_each_candle);

    fn collect_one_at(&self, index: usize) -> Option<OHLCCents> {
        let mapping = self.first_heights.load();
        let boundaries = mapping.starts();
        let price_len = self.prices.visible_len();
        let first = boundaries.get(index)?.to_usize().min(price_len);
        let end = boundaries
            .get(index + 1)
            .map_or(price_len, |height| height.to_usize().min(price_len));
        Some(self.candle(first, end))
    }

    /// Each run of adjacent requested candles shares one read; gaps are skipped.
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<OHLCCents>) {
        out.reserve(indices.len());
        for run in indices.chunk_by(|a, b| b == &(a + 1)) {
            self.for_each_candle(run[0], run[run.len() - 1] + 1, |candle| out.push(candle));
        }
    }
}

impl<I: VecIndex> Traversable for LazyOhlcVec<I> {
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
