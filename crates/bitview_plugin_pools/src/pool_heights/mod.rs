use std::{marker::PhantomData, sync::Arc};

use arc_swap::ArcSwap;
use bitview_primitives::PoolSlug;
use brk_types::Height;
use rangeindex::Pages;
use rustc_hash::FxHashMap;
use vecdb::{AnyVec, BytesVec, ReadOnlyClone, ReadableVec, TypedVec, VecIndex, VecValue, Version};

/// Which running total a lazy vec reads: the pool's block count, or its part of a chain-wide
/// per-block series (each non-`Blocks` discriminant identifies a slot in [`Totals`]).
#[derive(Clone, Copy)]
pub enum Column {
    Blocks,
    CoinbaseSats,
    CoinbaseCents,
    FeeSats,
    FeeCents,
    EmptyBlocks,
    Bip54LocktimeBlocks,
    NonzeroLocktimeBlocks,
    Under0p1SatVbTxs,
}

impl Column {
    pub(crate) const SUMMED: [Self; TOTALS] = [
        Self::CoinbaseSats,
        Self::CoinbaseCents,
        Self::FeeSats,
        Self::FeeCents,
        Self::EmptyBlocks,
        Self::Bip54LocktimeBlocks,
        Self::NonzeroLocktimeBlocks,
        Self::Under0p1SatVbTxs,
    ];
}

/// One block's value of each chain-wide series split by pool: one per [`Column`] after `Blocks`.
pub type Totals = [u64; TOTALS];

pub(crate) const TOTALS: usize = Column::Under0p1SatVbTxs as usize;
/// The amounts lead the summed columns; the rest are counts.
const AMOUNTS: usize = Column::FeeCents as usize;

/// A pool's running totals through one of its blocks. Its counts of blocks or transactions stay
/// far below 2^32, so they are kept in 32 bits: every pool's rows are in memory, one per block.
#[derive(Clone, Copy, Default)]
struct Running {
    amounts: [u64; AMOUNTS],
    counts: [u32; TOTALS - AMOUNTS],
}

const _: () = assert!(size_of::<Running>() == 48, "four amounts and four counts: 48 bytes a block");

impl Running {
    fn get(&self, slot: usize) -> u64 {
        match slot.checked_sub(AMOUNTS) {
            None => self.amounts[slot],
            Some(count) => u64::from(self.counts[count]),
        }
    }

    fn add(&mut self, block: Totals) {
        let (amounts, counts) = block.split_at(AMOUNTS);
        for (total, value) in self.amounts.iter_mut().zip(amounts) {
            *total += value;
        }
        for (total, value) in self.counts.iter_mut().zip(counts) {
            *total = u32::try_from(u64::from(*total) + value)
                .expect("a pool's running count fits 32 bits");
        }
    }
}

#[derive(Clone, Default)]
struct Record {
    heights: Pages<Height>,
    /// Running totals through each of `heights`, for the first `totals.len()` of them.
    totals: Pages<Running>,
}

static NO_HEIGHTS: Pages<Height> = Pages::new();

impl Record {
    fn heights(record: Option<&Self>) -> &Pages<Height> {
        record.map_or(&NO_HEIGHTS, |record| &record.heights)
    }
}

/// One published state of every pool's blocks and running totals. A clone shares every record
/// and page; the compute stages one, updates it, and publishes it whole.
#[derive(Clone, Default)]
pub(crate) struct PoolSnapshot {
    by_pool: FxHashMap<PoolSlug, Arc<Record>>,
    len: usize,
    version: Version,
    /// Blocks below this height are in every pool's totals.
    totals_len: usize,
    totals_version: Version,
}

/// Every pool's blocks and running totals, in memory; all per-pool series read it.
/// Each read loads one immutable snapshot, so readers never wait for the compute.
#[derive(Clone, Default)]
pub struct PoolHeights(Arc<ArcSwap<PoolSnapshot>>);

impl PoolHeights {
    /// Totals are filled by the first compute, once their sources are computed.
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
        Self(Arc::new(ArcSwap::from_pointee(PoolSnapshot {
            by_pool: by_pool
                .into_iter()
                .map(|(slug, record)| (slug, Arc::new(record)))
                .collect(),
            len,
            version: pool.version(),
            totals_len: 0,
            totals_version: Version::ZERO,
        })))
    }

    /// The published snapshot, to update privately and then [`Self::publish`].
    pub(crate) fn stage(&self) -> PoolSnapshot {
        PoolSnapshot::clone(&self.0.load())
    }

    pub(crate) fn publish(&self, snapshot: PoolSnapshot) {
        self.0.store(Arc::new(snapshot));
    }

    pub fn block_numbers(&self, slugs: &[PoolSlug], first_height: Height) -> Vec<u64> {
        let snapshot = self.0.load();
        let first_height = first_height.to_usize();

        slugs
            .iter()
            .enumerate()
            .map(|(offset, slug)| {
                snapshot.by_pool.get(slug).map_or(0, |record| {
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
        let snapshot = self.0.load();
        let Some(record) = snapshot.by_pool.get(&slug) else {
            return Vec::new();
        };
        let end = Self::cumulative_count(&record.heights, through_height.to_usize());
        let start = end.saturating_sub(limit);
        (start..end).rev().map(|i| record.heights[i]).collect()
    }

    fn cumulative_count(heights: &Pages<Height>, through_height: usize) -> usize {
        heights.partition_point(|height| height.to_usize() <= through_height)
    }
}

impl PoolSnapshot {
    pub(crate) fn update(&mut self, min: usize, version: Version, slugs: &[PoolSlug]) {
        // Only a rewind sweeps every pool; an append touches the new blocks' pools alone.
        if min < self.len {
            let totals_len = self.totals_len.min(min);
            for record in self.by_pool.values_mut() {
                let cut = record.heights.partition_point(|h| h.to_usize() < min);
                let filled = record
                    .heights
                    .partition_point(|h| h.to_usize() < totals_len);
                if cut < record.heights.len() || filled < record.totals.len() {
                    let record = Arc::make_mut(record);
                    record.heights.truncate(cut);
                    record.totals.truncate(filled);
                }
            }
            self.totals_len = totals_len;
        }
        debug_assert!(
            min <= self.len,
            "pool blocks resume within the known blocks"
        );
        self.len = min + slugs.len();
        self.version = version;
        for (offset, &slug) in slugs.iter().enumerate() {
            Arc::make_mut(self.by_pool.entry(slug).or_default())
                .heights
                .push(Height::from(min + offset));
        }
    }

    /// Drops every total from the height where they must resume, `to` at most, and returns it:
    /// their end under `version`, else zero.
    pub(crate) fn resume_totals(&mut self, version: Version, to: usize) -> usize {
        let from = if self.totals_version == version {
            self.totals_len.min(to)
        } else {
            0
        };
        // Only a refill sweeps every pool; resuming at the end keeps every total.
        if from < self.totals_len {
            for record in self.by_pool.values_mut() {
                let filled = record.heights.partition_point(|h| h.to_usize() < from);
                if filled < record.totals.len() {
                    Arc::make_mut(record).totals.truncate(filled);
                }
            }
        }
        self.totals_len = from;
        self.totals_version = version;
        from
    }

    /// Folds the next blocks (each one's pool and values) into their pools' running totals.
    pub(crate) fn append_totals(&mut self, blocks: impl IntoIterator<Item = (PoolSlug, Totals)>) {
        let mut height = self.totals_len;
        for (slug, block) in blocks {
            let record = Arc::make_mut(self.by_pool.get_mut(&slug).expect("block's pool is known"));
            debug_assert_eq!(
                record.heights.get(record.totals.len()),
                Some(&Height::from(height)),
                "totals fold in block order"
            );
            let mut running = record.totals.last().copied().unwrap_or_default();
            running.add(block);
            record.totals.push(running);
            height += 1;
        }
        debug_assert!(height <= self.len, "totals only for known blocks");
        self.totals_len = height;
    }
}

/// A pool's running total of one column, lazily read from `PoolHeights`.
#[derive(Clone)]
pub(crate) struct PoolCumulativeVec<T> {
    name: Arc<str>,
    slug: PoolSlug,
    column: Column,
    pool_heights: PoolHeights,
    value: PhantomData<fn() -> T>,
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

    fn len_of(&self, snapshot: &PoolSnapshot) -> usize {
        match self.column {
            Column::Blocks => snapshot.len,
            _ => snapshot.totals_len,
        }
    }

    /// The total through the first `position` of the pool's blocks.
    fn total(&self, record: Option<&Record>, position: usize) -> T {
        let value = match (self.column, record) {
            (Column::Blocks, _) => position as u64,
            (_, None) => 0,
            (column, Some(record)) => {
                debug_assert!(position <= record.totals.len(), "totals behind len");
                position
                    .checked_sub(1)
                    .map_or(0, |last| record.totals[last].get(column as usize - 1))
            }
        };
        T::from(value)
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let Ok(()) = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, !>(())
        });
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(T) -> Result<(), E>,
    ) -> Result<(), E> {
        let snapshot = self.pool_heights.0.load();
        let to = to.min(self.len_of(&snapshot));
        if from >= to {
            return Ok(());
        }
        let record = snapshot.by_pool.get(&self.slug).map(Arc::as_ref);
        let heights = Record::heights(record);
        let mut position = heights.partition_point(|height| height.to_usize() < from);
        let mut upcoming = heights.iter(position..heights.len()).peekable();
        for height in from..to {
            // A pool has at most one block per height.
            if upcoming
                .next_if(|pool_height| pool_height.to_usize() == height)
                .is_some()
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
        let snapshot = self.pool_heights.0.load();
        match self.column {
            Column::Blocks => snapshot.version,
            _ => snapshot.version + snapshot.totals_version,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.len_of(&self.pool_heights.0.load())
    }

    /// Each block's pool is fixed once mined: rows only change by a rollback.
    fn is_mutable(&self) -> bool {
        false
    }

    vecdb::any_vec_type_methods!(Height, T);
}

impl<T: VecValue + From<u64>> TypedVec for PoolCumulativeVec<T> {
    type I = Height;
    type T = T;
}

impl<T: VecValue + From<u64>> ReadableVec<Height, T> for PoolCumulativeVec<T> {
    vecdb::visitor_read_methods!(T, for_each_value, try_for_each_value);

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let snapshot = self.pool_heights.0.load();
        if index >= self.len_of(&snapshot) {
            return None;
        }
        let record = snapshot.by_pool.get(&self.slug).map(Arc::as_ref);
        let heights = Record::heights(record);
        Some(self.total(record, PoolHeights::cumulative_count(heights, index)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let Some(&first) = indices.first() else {
            return;
        };
        let snapshot = self.pool_heights.0.load();
        let len = self.len_of(&snapshot);
        let record = snapshot.by_pool.get(&self.slug).map(Arc::as_ref);
        let heights = Record::heights(record);
        let mut position = heights.partition_point(|height| height.to_usize() < first);
        // Lowest height not yet counted; a pool has at most one block per height, so at most
        // `index + 1 - next` of the following entries can be counted for `index`.
        let mut next = first;
        out.reserve(indices.len());
        for &index in indices {
            if index >= len {
                break;
            }
            let window = (index + 1)
                .saturating_sub(next)
                .min(heights.len() - position);
            position = heights.partition_point_in(position..position + window, |height| {
                height.to_usize() <= index
            });
            next = next.max(index + 1);
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
