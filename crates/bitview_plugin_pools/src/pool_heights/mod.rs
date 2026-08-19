use std::sync::Arc;

use brk_types::{Height, PoolSlug, StoredU64};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use vecdb::{
    AnyVec, BytesVec, PrintableIndex, ReadOnlyClone, ReadableVec, TypedVec, VecIndex, Version,
    short_type_name,
};

mod blocks_mined;

pub use blocks_mined::BlocksMined;

#[derive(Default)]
struct State {
    by_pool: FxHashMap<PoolSlug, Vec<Height>>,
    len: usize,
    version: Version,
}

#[derive(Clone, Default)]
pub struct PoolHeights(Arc<RwLock<State>>);

impl PoolHeights {
    pub fn build(pool: &BytesVec<Height, PoolSlug>) -> Self {
        let len = pool.len();
        let mut by_pool: FxHashMap<PoolSlug, Vec<Height>> = FxHashMap::default();
        let reader = pool.reader();
        for h in 0..len {
            by_pool
                .entry(reader.get_at(h))
                .or_default()
                .push(Height::from(h));
        }
        Self(Arc::new(RwLock::new(State {
            by_pool,
            len,
            version: pool.version(),
        })))
    }

    pub fn truncate(&self, min: usize) {
        let mut state = self.0.write();
        for heights in state.by_pool.values_mut() {
            let cut = heights.partition_point(|h| h.to_usize() < min);
            heights.truncate(cut);
        }
        state.len = min;
    }

    pub fn push(&self, slug: PoolSlug, height: Height) {
        let mut state = self.0.write();
        debug_assert_eq!(height.to_usize(), state.len);
        state.by_pool.entry(slug).or_default().push(height);
        state.len = height.to_usize() + 1;
    }

    pub fn block_numbers(&self, slugs: &[PoolSlug], first_height: Height) -> Vec<u64> {
        let state = self.0.read();
        let first_height = first_height.to_usize();

        slugs
            .iter()
            .enumerate()
            .map(|(offset, slug)| {
                state.by_pool.get(slug).map_or(0, |heights| {
                    Self::cumulative_count(heights, first_height + offset) as u64
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
        let Some(heights) = state.by_pool.get(&slug) else {
            return Vec::new();
        };
        let end = Self::cumulative_count(heights, through_height.to_usize());
        let start = end.saturating_sub(limit);
        heights[start..end].iter().rev().copied().collect()
    }

    pub fn latest_height(&self, slug: PoolSlug, through_height: Height) -> Option<Height> {
        let state = self.0.read();
        let heights = state.by_pool.get(&slug)?;
        let end = Self::cumulative_count(heights, through_height.to_usize());
        end.checked_sub(1).map(|index| heights[index])
    }

    fn cumulative_count(heights: &[Height], through_height: usize) -> usize {
        heights.partition_point(|height| height.to_usize() <= through_height)
    }

    fn len(&self) -> usize {
        self.0.read().len
    }

    fn version(&self) -> Version {
        self.0.read().version
    }
}

/// Lazy cumulative block count for one pool, backed by the shared sorted
/// `PoolHeights` cache.
#[derive(Clone)]
struct PoolCumulativeVec {
    name: Arc<str>,
    slug: PoolSlug,
    pool_heights: PoolHeights,
}

impl PoolCumulativeVec {
    fn new(name: &str, slug: PoolSlug, pool_heights: PoolHeights) -> Self {
        Self {
            name: Arc::from(name),
            slug,
            pool_heights,
        }
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(StoredU64)) {
        let result = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, std::convert::Infallible>(())
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
        mut each: impl FnMut(StoredU64) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        if from >= to {
            return Ok(());
        }

        let state = self.pool_heights.0.read();
        let heights = state.by_pool.get(&self.slug).map_or(&[][..], Vec::as_slice);
        let mut position = heights.partition_point(|height| height.to_usize() < from);
        let mut cumulative = position as u64;

        for height in from..to {
            while heights
                .get(position)
                .is_some_and(|pool_height| pool_height.to_usize() == height)
            {
                position += 1;
                cumulative += 1;
            }
            each(StoredU64::from(cumulative))?;
        }
        Ok(())
    }
}

impl AnyVec for PoolCumulativeVec {
    fn version(&self) -> Version {
        self.pool_heights.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.pool_heights.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<StoredU64>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<StoredU64>()
    }
}

impl TypedVec for PoolCumulativeVec {
    type I = Height;
    type T = StoredU64;
}

impl ReadableVec<Height, StoredU64> for PoolCumulativeVec {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<StoredU64>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(StoredU64)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, StoredU64) -> B>(
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

    fn try_fold_range_at<B, E, F: FnMut(B, StoredU64) -> Result<B, E>>(
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

    fn collect_one_at(&self, index: usize) -> Option<StoredU64> {
        if index >= self.len() {
            return None;
        }
        let state = self.pool_heights.0.read();
        let heights = state.by_pool.get(&self.slug).map_or(&[][..], Vec::as_slice);
        Some(StoredU64::from(
            PoolHeights::cumulative_count(heights, index) as u64,
        ))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<StoredU64>) {
        let Some(&first) = indices.first() else {
            return;
        };
        let len = self.len();
        if first >= len {
            return;
        }

        let state = self.pool_heights.0.read();
        let heights = state.by_pool.get(&self.slug).map_or(&[][..], Vec::as_slice);
        let mut position = PoolHeights::cumulative_count(heights, first);

        out.reserve(indices.len());
        out.push(StoredU64::from(position as u64));
        for &index in &indices[1..] {
            if index >= len {
                break;
            }
            while heights
                .get(position)
                .is_some_and(|height| height.to_usize() <= index)
            {
                position += 1;
            }
            out.push(StoredU64::from(position as u64));
        }
    }
}

impl ReadOnlyClone for PoolCumulativeVec {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use vecdb::ReadableVec;

    use super::*;

    fn fixture() -> (PoolHeights, PoolCumulativeVec) {
        let pool_heights = PoolHeights::default();
        for (height, slug) in [
            PoolSlug::F2Pool,
            PoolSlug::Unknown,
            PoolSlug::F2Pool,
            PoolSlug::F2Pool,
            PoolSlug::Unknown,
        ]
        .into_iter()
        .enumerate()
        {
            pool_heights.push(slug, Height::from(height));
        }
        let cumulative = PoolCumulativeVec::new(
            "f2pool_blocks_mined_cumulative",
            PoolSlug::F2Pool,
            pool_heights.clone(),
        );
        (pool_heights, cumulative)
    }

    #[test]
    fn cumulative_counts_include_the_current_height() {
        let (_, cumulative) = fixture();

        assert_eq!(
            cumulative.collect(),
            [1_u64, 1, 2, 3, 3].map(StoredU64::from)
        );
        assert_eq!(
            cumulative.collect_range_at(1, 4),
            [1_u64, 2, 3].map(StoredU64::from)
        );
        assert_eq!(cumulative.collect_one_at(4), Some(StoredU64::from(3_u64)));
        assert_eq!(cumulative.collect_one_at(5), None);
    }

    #[test]
    fn sorted_and_appending_reads_use_cumulative_ranks() {
        let (_, cumulative) = fixture();
        let mut sorted = vec![StoredU64::from(99_u64)];
        cumulative.read_sorted_into_at(&[0, 2, 4], &mut sorted);
        assert_eq!(sorted, [99_u64, 1, 2, 3].map(StoredU64::from));

        let mut range = vec![StoredU64::from(99_u64)];
        cumulative.read_into_at(2, 5, &mut range);
        assert_eq!(range, [99_u64, 2, 3, 3].map(StoredU64::from));
    }

    #[test]
    fn truncate_and_push_keep_length_and_counts_in_sync() {
        let (pool_heights, cumulative) = fixture();

        pool_heights.truncate(3);
        assert_eq!(cumulative.collect(), [1_u64, 1, 2].map(StoredU64::from));

        pool_heights.push(PoolSlug::Unknown, Height::from(3_u32));
        pool_heights.push(PoolSlug::F2Pool, Height::from(4_u32));
        assert_eq!(
            cumulative.collect(),
            [1_u64, 1, 2, 2, 3].map(StoredU64::from)
        );
    }

    #[test]
    fn absent_pool_is_zero_for_every_height() {
        let (pool_heights, _) = fixture();
        let cumulative = PoolCumulativeVec::new("absent", PoolSlug::Luxor, pool_heights);

        assert_eq!(cumulative.collect(), [StoredU64::from(0_u64); 5]);
    }

    #[test]
    fn bulk_block_numbers_use_one_height_per_slug() {
        let (pool_heights, _) = fixture();

        assert_eq!(
            pool_heights.block_numbers(
                &[PoolSlug::F2Pool, PoolSlug::Unknown, PoolSlug::F2Pool],
                Height::from(0_u32),
            ),
            [1, 1, 2]
        );
    }

    #[test]
    fn latest_heights_are_limited_and_descending() {
        let (pool_heights, _) = fixture();

        assert_eq!(
            pool_heights.latest_heights(PoolSlug::F2Pool, Height::from(4_u32), 2),
            [Height::from(3_u32), Height::from(2_u32)]
        );
        assert_eq!(
            pool_heights.latest_heights(PoolSlug::F2Pool, Height::from(2_u32), 10),
            [Height::from(2_u32), Height::from(0_u32)]
        );
    }

    #[test]
    fn latest_height_is_bounded_by_height() {
        let (pool_heights, _) = fixture();

        assert_eq!(
            pool_heights.latest_height(PoolSlug::F2Pool, Height::from(4_u32)),
            Some(Height::from(3_u32))
        );
        assert_eq!(
            pool_heights.latest_height(PoolSlug::F2Pool, Height::from(1_u32)),
            Some(Height::from(0_u32))
        );
        assert_eq!(
            pool_heights.latest_height(PoolSlug::Luxor, Height::from(4_u32)),
            None
        );
    }
}
