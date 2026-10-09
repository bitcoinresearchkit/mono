use std::{convert::Infallible, marker::PhantomData, sync::Arc};

use bitview_primitives::PoolSlug;
use brk_types::Height;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use vecdb::{
    AnyVec, BytesVec, PrintableIndex, ReadOnlyClone, ReadableVec, TypedVec, VecIndex, VecValue,
    Version, short_type_name,
};

/// Per-block reward values of one block: coinbase and fees, in sats and in cents at the
/// block's spot price.
pub type Rewards = [u64; 4];

/// Which running total a lazy vec reads.
#[derive(Clone, Copy)]
pub enum Column {
    Blocks,
    CoinbaseSats,
    CoinbaseCents,
    FeeSats,
    FeeCents,
}

#[derive(Default)]
struct Record {
    heights: Vec<Height>,
    /// Running reward totals through each of `heights`, for the first `totals.len()` of them.
    totals: Vec<Rewards>,
}

#[derive(Default)]
struct State {
    by_pool: FxHashMap<PoolSlug, Record>,
    len: usize,
    version: Version,
    /// Blocks below this height have their rewards in every pool's totals.
    totals_len: usize,
    totals_version: Version,
}

/// Every pool's blocks and reward running totals, in memory; all per-pool series read it.
#[derive(Clone, Default)]
pub struct PoolHeights(Arc<RwLock<State>>);

impl PoolHeights {
    /// Rewards are filled by the first compute, once their sources are computed.
    pub(crate) fn build(pool: &BytesVec<Height, PoolSlug>) -> Self {
        let len = pool.len();
        let mut by_pool: FxHashMap<PoolSlug, Record> = FxHashMap::default();
        let reader = pool.reader();
        for h in 0..len {
            by_pool
                .entry(reader.get_at(h))
                .or_default()
                .heights
                .push(Height::from(h));
        }
        Self(Arc::new(RwLock::new(State {
            by_pool,
            len,
            version: pool.version(),
            totals_len: 0,
            totals_version: Version::ZERO,
        })))
    }

    pub(crate) fn update(&self, min: usize, version: Version, slugs: &[PoolSlug]) {
        let mut state = self.0.write();
        let totals_len = state.totals_len.min(min);
        for record in state.by_pool.values_mut() {
            let cut = record.heights.partition_point(|h| h.to_usize() < min);
            record.heights.truncate(cut);
            let filled = record
                .heights
                .partition_point(|h| h.to_usize() < totals_len);
            record.totals.truncate(filled);
        }
        state.len = min;
        state.totals_len = totals_len;
        state.version = version;
        for (offset, &slug) in slugs.iter().enumerate() {
            state
                .by_pool
                .entry(slug)
                .or_default()
                .heights
                .push(Height::from(min + offset));
        }
        state.len += slugs.len();
    }

    /// Height where reward totals must resume; everything from it is (re)filled.
    pub(crate) fn totals_start(&self, version: Version) -> usize {
        let state = self.0.read();
        if state.totals_version == version {
            state.totals_len
        } else {
            0
        }
    }

    /// `rewards[i]` belongs to block `from + i`; totals resume at `from` (see `totals_start`).
    pub(crate) fn fill_totals(&self, from: usize, rewards: &[Rewards], version: Version) {
        let mut state = self.0.write();
        let end = (from + rewards.len()).min(state.len);
        for record in state.by_pool.values_mut() {
            let filled = record.heights.partition_point(|h| h.to_usize() < from);
            record.totals.truncate(filled);
            let mut running = record.totals.last().copied().unwrap_or_default();
            for height in &record.heights[filled..] {
                let height = height.to_usize();
                if height >= end {
                    break;
                }
                let block = rewards[height - from];
                for (total, value) in running.iter_mut().zip(block) {
                    *total += value;
                }
                record.totals.push(running);
            }
        }
        state.totals_len = end;
        state.totals_version = version;
    }

    pub fn block_numbers(&self, slugs: &[PoolSlug], first_height: Height) -> Vec<u64> {
        let state = self.0.read();
        let first_height = first_height.to_usize();

        slugs
            .iter()
            .enumerate()
            .map(|(offset, slug)| {
                state.by_pool.get(slug).map_or(0, |record| {
                    Self::cumulative_count(&record.heights, first_height + offset) as u64
                })
            })
            .collect()
    }

    pub fn latest_heights(
        &self,
        slug: PoolSlug,
        through_height: Height,
        limit: usize,
    ) -> Vec<Height> {
        let state = self.0.read();
        let Some(record) = state.by_pool.get(&slug) else {
            return Vec::new();
        };
        let end = Self::cumulative_count(&record.heights, through_height.to_usize());
        let start = end.saturating_sub(limit);
        record.heights[start..end].iter().rev().copied().collect()
    }

    fn cumulative_count(heights: &[Height], through_height: usize) -> usize {
        heights.partition_point(|height| height.to_usize() <= through_height)
    }
}

/// A pool's running total of one column, lazily read from `PoolHeights`.
pub(crate) struct PoolCumulativeVec<T> {
    name: Arc<str>,
    slug: PoolSlug,
    column: Column,
    pool_heights: PoolHeights,
    value: PhantomData<fn() -> T>,
}

impl<T> Clone for PoolCumulativeVec<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            slug: self.slug,
            column: self.column,
            pool_heights: self.pool_heights.clone(),
            value: PhantomData,
        }
    }
}

impl<T: VecValue + From<u64>> PoolCumulativeVec<T> {
    pub(crate) fn new(
        name: &str,
        slug: PoolSlug,
        column: Column,
        pool_heights: PoolHeights,
    ) -> Self {
        Self {
            name: Arc::from(name),
            slug,
            column,
            pool_heights,
            value: PhantomData,
        }
    }

    /// The total through the first `position` of the pool's blocks.
    fn total(&self, record: Option<&Record>, position: usize) -> T {
        let value = match (self.column, record) {
            (Column::Blocks, _) => position as u64,
            (_, None) => 0,
            (column, Some(record)) => {
                debug_assert!(position <= record.totals.len(), "reward totals behind len");
                position
                    .checked_sub(1)
                    .and_then(|last| record.totals.get(last))
                    .map_or(0, |totals| totals[column as usize - 1])
            }
        };
        T::from(value)
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let result = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(T) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        if from >= to {
            return Ok(());
        }
        let state = self.pool_heights.0.read();
        let record = state.by_pool.get(&self.slug);
        let heights = record.map_or(&[][..], |record| record.heights.as_slice());
        let mut position = heights.partition_point(|height| height.to_usize() < from);
        for height in from..to {
            while heights
                .get(position)
                .is_some_and(|pool_height| pool_height.to_usize() == height)
            {
                position += 1;
            }
            each(self.total(record, position))?;
        }
        Ok(())
    }
}

impl<T: VecValue + From<u64>> AnyVec for PoolCumulativeVec<T> {
    fn version(&self) -> Version {
        let state = self.pool_heights.0.read();
        match self.column {
            Column::Blocks => state.version,
            _ => state.version + state.totals_version,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        let state = self.pool_heights.0.read();
        match self.column {
            Column::Blocks => state.len,
            _ => state.totals_len.min(state.len),
        }
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }
}

impl<T: VecValue + From<u64>> TypedVec for PoolCumulativeVec<T> {
    type I = Height;
    type T = T;
}

impl<T: VecValue + From<u64>> ReadableVec<Height, T> for PoolCumulativeVec<T> {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        let state = self.pool_heights.0.read();
        let record = state.by_pool.get(&self.slug);
        let heights = record.map_or(&[][..], |record| record.heights.as_slice());
        Some(self.total(record, PoolHeights::cumulative_count(heights, index)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let Some(&first) = indices.first() else {
            return;
        };
        let len = self.len();
        let state = self.pool_heights.0.read();
        let record = state.by_pool.get(&self.slug);
        let heights = record.map_or(&[][..], |record| record.heights.as_slice());
        let mut position = heights.partition_point(|height| height.to_usize() < first);
        out.reserve(indices.len());
        for &index in indices {
            if index >= len {
                break;
            }
            while heights
                .get(position)
                .is_some_and(|height| height.to_usize() <= index)
            {
                position += 1;
            }
            out.push(self.total(record, position));
        }
    }
}

impl<T: VecValue + From<u64>> ReadOnlyClone for PoolCumulativeVec<T> {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}
