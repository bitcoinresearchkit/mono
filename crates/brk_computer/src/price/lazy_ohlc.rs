use std::sync::Arc;

use brk_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Cents, Close, Height, High, Low, OHLCCents, Open, Version};
use vecdb::{
    AnyExportableVec, AnyVec, CachedBoxedVec, ReadableVec, TypedVec, VecIndex, short_type_name,
};

use crate::indexes::CachedFirstHeightVec;

/// OHLC candles derived directly from pinned spot prices and period boundaries.
#[derive(Clone)]
pub struct LazyOhlcVec<I: VecIndex> {
    name: Arc<str>,
    base_version: Version,
    prices: CachedBoxedVec<Height, Cents>,
    first_heights: CachedFirstHeightVec<I>,
}

impl<I: VecIndex> LazyOhlcVec<I> {
    pub(crate) fn new(
        name: &str,
        version: Version,
        prices: CachedBoxedVec<Height, Cents>,
        first_heights: CachedFirstHeightVec<I>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            prices,
            first_heights,
        }
    }

    fn try_for_each_candle<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(OHLCCents) -> Result<(), E>,
    ) -> Result<(), E> {
        let first_heights = self.first_heights.snapshot();
        let to = to.min(first_heights.len());
        if from >= to {
            return Ok(());
        }

        let prices = self.prices.snapshot();
        for index in from..to {
            each(Self::candle_at(index, &prices, &first_heights).unwrap())?;
        }

        Ok(())
    }

    fn for_each_candle(&self, from: usize, to: usize, mut each: impl FnMut(OHLCCents)) {
        let result = self.try_for_each_candle(from, to, |candle| {
            each(candle);
            Ok::<_, std::convert::Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }

    fn candle_at(index: usize, prices: &[Cents], first_heights: &[Height]) -> Option<OHLCCents> {
        let first = first_heights.get(index)?.to_usize().min(prices.len());
        let end = first_heights
            .get(index + 1)
            .map_or(prices.len(), |height| height.to_usize().min(prices.len()));

        if first < end {
            let mut candle = CandleBuilder::new(prices[first]);
            for &price in &prices[first + 1..end] {
                candle.include(price);
            }
            Some(candle.finish())
        } else {
            let close = first
                .checked_sub(1)
                .and_then(|height| prices.get(height))
                .copied()
                .unwrap_or_default();
            Some(OHLCCents::from(Close::new(close)))
        }
    }
}

impl<I: VecIndex> AnyVec for LazyOhlcVec<I> {
    fn version(&self) -> Version {
        self.base_version + self.prices.version() + self.first_heights.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.first_heights.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
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
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<OHLCCents>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_candle(from, to, |candle| buf.push(candle));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(OHLCCents)) {
        self.for_each_candle(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, OHLCCents) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_candle(from, to, |candle| {
            acc = Some(fold(acc.take().unwrap(), candle));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, OHLCCents) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_candle(from, to, |candle| {
            acc = Some(fold(acc.take().unwrap(), candle)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<OHLCCents> {
        let prices = self.prices.snapshot();
        let first_heights = self.first_heights.snapshot();
        Self::candle_at(index, &prices, &first_heights)
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<OHLCCents>) {
        let prices = self.prices.snapshot();
        let first_heights = self.first_heights.snapshot();
        out.reserve(indices.len());
        indices
            .iter()
            .filter_map(|&index| Self::candle_at(index, &prices, &first_heights))
            .for_each(|candle| out.push(candle));
    }
}

impl<I: VecIndex> Traversable for LazyOhlcVec<I> {
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, OHLCCents, _>(self)
    }
}

struct CandleBuilder {
    open: Cents,
    high: Cents,
    low: Cents,
    close: Cents,
}

impl CandleBuilder {
    fn new(price: Cents) -> Self {
        Self {
            open: price,
            high: price,
            low: price,
            close: price,
        }
    }

    fn include(&mut self, price: Cents) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
    }

    fn finish(self) -> OHLCCents {
        OHLCCents {
            open: Open::new(self.open),
            high: High::new(self.high),
            low: Low::new(self.low),
            close: Close::new(self.close),
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Day1;
    use vecdb::{
        AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec,
        WritableVec,
    };

    use super::*;

    fn values(candle: &OHLCCents) -> (u64, u64, u64, u64) {
        (**candle.open, **candle.high, **candle.low, **candle.close)
    }

    #[test]
    fn derives_candles_and_preserves_empty_periods() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("brk-lazy-ohlc-{}-{suffix}", std::process::id()));
        let db = Database::open(&path).unwrap();

        let mut prices: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "prices", Version::ONE).unwrap();
        let mut periods: EagerVec<PcoVec<Height, Day1>> =
            EagerVec::forced_import(&db, "periods", Version::ONE).unwrap();

        for value in [10, 20, 5, 7] {
            prices.push(Cents::new(value));
        }
        for period in [0, 2, 2, 4] {
            periods.push(Day1::from(period));
        }
        prices.write().unwrap();
        periods.write().unwrap();

        let prices = CachedVec::wrap(prices);
        let first_heights = CachedFirstHeightVec::new(periods.read_only_boxed_clone());
        let ohlc = LazyOhlcVec::new(
            "ohlc",
            Version::ONE,
            prices.read_only_cached_boxed_clone(),
            first_heights,
        );

        let candles = ohlc.collect();
        assert_eq!(candles.len(), 5);
        assert_eq!(
            candles.iter().map(values).collect::<Vec<_>>(),
            [
                (10, 10, 10, 10),
                (10, 10, 10, 10),
                (20, 20, 5, 5),
                (5, 5, 5, 5),
                (7, 7, 7, 7)
            ],
        );
        assert_eq!(
            ohlc.collect_range(Day1::from(1), Day1::from(4))
                .iter()
                .map(values)
                .collect::<Vec<_>>(),
            candles[1..4].iter().map(values).collect::<Vec<_>>(),
        );
        assert_eq!(
            ohlc.collect_one(Day1::from(2)).as_ref().map(values),
            Some(values(&candles[2])),
        );

        drop(ohlc);
        drop(prices);
        drop(periods);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
