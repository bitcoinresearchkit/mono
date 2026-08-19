use brk_error::Result;

use std::collections::VecDeque;

use bitview_traversable::Traversable;
use brk_types::{Height, StoredU64, VSize, get_percentile, get_weighted_percentile};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, CheckedSub, Database, Exit, ReadableVec, Rw, StorageMode, VecIndex,
    VecValue, Version, WritableVec,
};

use crate::{ComputedVecValue, DistributionStats, NumericValue, PerBlock};

fn effective_range(first: usize, count: usize, skip_count: usize) -> std::ops::Range<usize> {
    let start = first + skip_count.min(count);
    start..first + count
}

fn merge_sorted<T: Copy + Ord>(window: &mut Vec<T>, block: &[T], buffer: &mut Vec<T>) {
    buffer.clear();
    buffer.reserve(window.len() + block.len());

    let (mut wi, mut bi) = (0, 0);
    while wi < window.len() && bi < block.len() {
        if window[wi] <= block[bi] {
            buffer.push(window[wi]);
            wi += 1;
        } else {
            buffer.push(block[bi]);
            bi += 1;
        }
    }
    buffer.extend_from_slice(&window[wi..]);
    buffer.extend_from_slice(&block[bi..]);
    std::mem::swap(window, buffer);
}

fn remove_sorted<T: Copy + Ord>(window: &mut Vec<T>, block: &[T], buffer: &mut Vec<T>) {
    buffer.clear();
    buffer.reserve(window.len().saturating_sub(block.len()));

    let mut bi = 0;
    for &value in window.iter() {
        if bi < block.len() && value == block[bi] {
            bi += 1;
        } else {
            buffer.push(value);
        }
    }
    debug_assert_eq!(bi, block.len());
    std::mem::swap(window, buffer);
}

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct PerBlockDistribution<T: ComputedVecValue + PartialOrd + JsonSchema, M: StorageMode = Rw>(
    pub DistributionStats<PerBlock<T, M>>,
);

impl<T: NumericValue + JsonSchema> PerBlockDistribution<T> {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|suffix| {
            PerBlock::forced_import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }

    pub fn compute_with_skip<A>(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<A, T>,
        first_indexes: &impl ReadableVec<Height, A>,
        count_indexes: &impl ReadableVec<Height, StoredU64>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        A: VecIndex + VecValue + CheckedSub<A>,
    {
        let DistributionStats {
            min,
            max,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
        } = &mut self.0;

        let min = &mut min.height;
        let max = &mut max.height;
        let pct10 = &mut pct10.height;
        let pct25 = &mut pct25.height;
        let median = &mut median.height;
        let pct75 = &mut pct75.height;
        let pct90 = &mut pct90.height;

        let combined_version = source.version() + first_indexes.version() + count_indexes.version();

        let mut index = max_from;
        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.validate_computed_version_or_reset(combined_version)?;
            index = index.min(Height::from(vec.len()));
        }

        let start = index.to_usize();

        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.truncate_if_needed_at(start)?;
        }

        let fi_len = first_indexes.len();
        let first_indexes_batch: Vec<A> = first_indexes.collect_range_at(start, fi_len);
        let count_indexes_batch: Vec<StoredU64> = count_indexes.collect_range_at(start, fi_len);

        let zero = T::from(0_usize);
        let mut values: Vec<T> = Vec::new();

        first_indexes_batch
            .into_iter()
            .zip(count_indexes_batch)
            .try_for_each(|(first_index, count_index)| -> Result<()> {
                let count = u64::from(count_index) as usize;
                let effective_count = count.saturating_sub(skip_count);
                let effective_first_index = first_index + skip_count.min(count);

                source.collect_range_into_at(
                    effective_first_index.to_usize(),
                    effective_first_index.to_usize() + effective_count,
                    &mut values,
                );

                if skip_count > 0 {
                    values.retain(|v| *v > zero);
                }

                if values.is_empty() {
                    for vec in [
                        &mut *min,
                        &mut *max,
                        &mut *median,
                        &mut *pct10,
                        &mut *pct25,
                        &mut *pct75,
                        &mut *pct90,
                    ] {
                        vec.push(zero);
                    }
                } else {
                    values.sort_unstable();

                    max.push(*values.last().unwrap());
                    pct90.push(get_percentile(&values, 0.90));
                    pct75.push(get_percentile(&values, 0.75));
                    median.push(get_percentile(&values, 0.50));
                    pct25.push(get_percentile(&values, 0.25));
                    pct10.push(get_percentile(&values, 0.10));
                    min.push(*values.first().unwrap());
                }

                Ok(())
            })?;

        let _lock = exit.lock();
        for vec in [min, max, median, pct10, pct25, pct75, pct90] {
            vec.write()?;
        }

        Ok(())
    }

    /// Like `compute_with_skip` but uses vsize-weighted percentiles.
    /// Each transaction's contribution to percentile rank is proportional to its vsize.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_with_skip_weighted<A>(
        &mut self,
        max_from: Height,
        source: &impl ReadableVec<A, T>,
        vsize_source: &impl ReadableVec<A, VSize>,
        first_indexes: &impl ReadableVec<Height, A>,
        count_indexes: &impl ReadableVec<Height, StoredU64>,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        A: VecIndex + VecValue + CheckedSub<A>,
    {
        let DistributionStats {
            min,
            max,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
        } = &mut self.0;

        let min = &mut min.height;
        let max = &mut max.height;
        let pct10 = &mut pct10.height;
        let pct25 = &mut pct25.height;
        let median = &mut median.height;
        let pct75 = &mut pct75.height;
        let pct90 = &mut pct90.height;

        let combined_version = source.version()
            + vsize_source.version()
            + first_indexes.version()
            + count_indexes.version();

        let mut index = max_from;
        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.validate_computed_version_or_reset(combined_version)?;
            index = index.min(Height::from(vec.len()));
        }

        let start = index.to_usize();

        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.truncate_if_needed_at(start)?;
        }

        let fi_len = first_indexes.len();
        let first_indexes_batch: Vec<A> = first_indexes.collect_range_at(start, fi_len);
        let count_indexes_batch: Vec<StoredU64> = count_indexes.collect_range_at(start, fi_len);

        let zero = T::from(0_usize);
        let mut values: Vec<T> = Vec::new();
        let mut vsizes: Vec<VSize> = Vec::new();
        let mut weighted: Vec<(T, VSize)> = Vec::new();

        first_indexes_batch
            .into_iter()
            .zip(count_indexes_batch)
            .try_for_each(|(first_index, count_index)| -> Result<()> {
                let count = u64::from(count_index) as usize;
                let effective_count = count.saturating_sub(skip_count);
                let effective_first_index = first_index + skip_count.min(count);

                let start_at = effective_first_index.to_usize();
                let end_at = start_at + effective_count;

                source.collect_range_into_at(start_at, end_at, &mut values);
                vsize_source.collect_range_into_at(start_at, end_at, &mut vsizes);

                weighted.clear();
                weighted.extend(
                    values
                        .iter()
                        .copied()
                        .zip(vsizes.iter().copied())
                        .filter(|(v, _)| skip_count == 0 || *v > zero),
                );

                if weighted.is_empty() {
                    for vec in [
                        &mut *min,
                        &mut *max,
                        &mut *median,
                        &mut *pct10,
                        &mut *pct25,
                        &mut *pct75,
                        &mut *pct90,
                    ] {
                        vec.push(zero);
                    }
                } else {
                    weighted.sort_unstable_by_key(|a| a.0);

                    max.push(weighted.last().unwrap().0);
                    pct90.push(get_weighted_percentile(&weighted, 0.90));
                    pct75.push(get_weighted_percentile(&weighted, 0.75));
                    median.push(get_weighted_percentile(&weighted, 0.50));
                    pct25.push(get_weighted_percentile(&weighted, 0.25));
                    pct10.push(get_weighted_percentile(&weighted, 0.10));
                    min.push(weighted.first().unwrap().0);
                }

                Ok(())
            })?;

        let _lock = exit.lock();
        for vec in [min, max, median, pct10, pct25, pct75, pct90] {
            vec.write()?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_from_nblocks<A>(
        &mut self,
        max_from: Height,
        source: &(impl ReadableVec<A, T> + Sized),
        first_indexes: &impl ReadableVec<Height, A>,
        count_indexes: &impl ReadableVec<Height, StoredU64>,
        n_blocks: usize,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        T: CheckedSub,
        A: VecIndex + VecValue + CheckedSub<A>,
    {
        assert!(n_blocks > 0);

        let DistributionStats {
            min,
            max,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
        } = &mut self.0;

        let min = &mut min.height;
        let max = &mut max.height;
        let pct10 = &mut pct10.height;
        let pct25 = &mut pct25.height;
        let median = &mut median.height;
        let pct75 = &mut pct75.height;
        let pct90 = &mut pct90.height;

        let combined_version = source.version() + first_indexes.version() + count_indexes.version();

        let mut index = max_from;
        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.validate_computed_version_or_reset(combined_version)?;
            index = index.min(Height::from(vec.len()));
        }

        let start = index.to_usize();
        let fi_len = first_indexes.len();

        let batch_start = start.saturating_sub(n_blocks - 1);
        let first_indexes_batch: Vec<A> = first_indexes.collect_range_at(batch_start, fi_len);
        let count_indexes_all: Vec<StoredU64> = count_indexes.collect_range_at(batch_start, fi_len);

        let zero = T::from(0_usize);

        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.truncate_if_needed_at(start)?;
        }

        // Persistent sorted window: O(n) merge-insert for new block, O(n) merge-filter
        // for expired block. Avoids re-sorting every block. Cursor reads only the new
        // block (~1 page decompress vs original's ~4). Ring buffer caches per-block
        // sorted values for O(1) expiry.
        // Peak memory: 2 × ~15k window elements + n_blocks × ~2500 cached ≈ 360 KB.
        let mut block_ring: VecDeque<Vec<T>> = VecDeque::with_capacity(n_blocks + 1);
        let mut cursor = source.cursor();
        let mut sorted_window: Vec<T> = Vec::new();
        let mut merge_buf: Vec<T> = Vec::new();

        // Pre-fill initial window blocks [window_start_of_first..start)
        let window_start_of_first = start.saturating_sub(n_blocks - 1);
        for block_idx in window_start_of_first..start {
            let fi = first_indexes_batch[block_idx - batch_start].to_usize();
            let count = u64::from(count_indexes_all[block_idx - batch_start]) as usize;
            let range = effective_range(fi, count, skip_count);
            if cursor.position() < range.start {
                cursor.advance(range.start - cursor.position());
            }
            let mut bv = Vec::with_capacity(range.len());
            cursor.for_each(range.len(), |v: T| {
                if skip_count == 0 || v > zero {
                    bv.push(v);
                }
            });
            bv.sort_unstable();
            sorted_window.extend_from_slice(&bv);
            block_ring.push_back(bv);
        }
        // Initial sorted_window was built by extending individually sorted blocks —
        // stable sort detects these sorted runs and merges in O(n × log(k)) instead of O(n log n).
        sorted_window.sort();

        for j in 0..(fi_len - start) {
            let idx = start + j;

            // Read and sort new block's values
            let fi = first_indexes_batch[idx - batch_start].to_usize();
            let count = u64::from(count_indexes_all[idx - batch_start]) as usize;
            let range = effective_range(fi, count, skip_count);
            if cursor.position() < range.start {
                cursor.advance(range.start - cursor.position());
            }
            let mut new_block = Vec::with_capacity(range.len());
            cursor.for_each(range.len(), |v: T| {
                if skip_count == 0 || v > zero {
                    new_block.push(v);
                }
            });
            new_block.sort_unstable();

            // Merge-insert new sorted block into sorted_window: O(n+m)
            merge_sorted(&mut sorted_window, &new_block, &mut merge_buf);

            block_ring.push_back(new_block);

            // Expire oldest block: merge-filter its sorted values from sorted_window in O(n)
            if block_ring.len() > n_blocks {
                let expired = block_ring.pop_front().unwrap();

                remove_sorted(&mut sorted_window, &expired, &mut merge_buf);
            }

            if sorted_window.is_empty() {
                for vec in [
                    &mut *min,
                    &mut *max,
                    &mut *median,
                    &mut *pct10,
                    &mut *pct25,
                    &mut *pct75,
                    &mut *pct90,
                ] {
                    vec.push(zero);
                }
            } else {
                max.push(*sorted_window.last().unwrap());
                pct90.push(get_percentile(&sorted_window, 0.90));
                pct75.push(get_percentile(&sorted_window, 0.75));
                median.push(get_percentile(&sorted_window, 0.50));
                pct25.push(get_percentile(&sorted_window, 0.25));
                pct10.push(get_percentile(&sorted_window, 0.10));
                min.push(*sorted_window.first().unwrap());
            }
        }

        let _lock = exit.lock();
        for vec in [min, max, median, pct10, pct25, pct75, pct90] {
            vec.write()?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_from_nblocks_weighted<A>(
        &mut self,
        max_from: Height,
        source: &(impl ReadableVec<A, T> + Sized),
        vsize_source: &(impl ReadableVec<A, VSize> + Sized),
        first_indexes: &impl ReadableVec<Height, A>,
        count_indexes: &impl ReadableVec<Height, StoredU64>,
        n_blocks: usize,
        exit: &Exit,
        skip_count: usize,
    ) -> Result<()>
    where
        T: CheckedSub,
        A: VecIndex + VecValue + CheckedSub<A>,
    {
        assert!(n_blocks > 0);

        let DistributionStats {
            min,
            max,
            pct10,
            pct25,
            median,
            pct75,
            pct90,
        } = &mut self.0;

        let min = &mut min.height;
        let max = &mut max.height;
        let pct10 = &mut pct10.height;
        let pct25 = &mut pct25.height;
        let median = &mut median.height;
        let pct75 = &mut pct75.height;
        let pct90 = &mut pct90.height;

        let combined_version = source.version()
            + vsize_source.version()
            + first_indexes.version()
            + count_indexes.version();

        let mut index = max_from;
        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.validate_computed_version_or_reset(combined_version)?;
            index = index.min(Height::from(vec.len()));
        }

        let start = index.to_usize();
        let fi_len = first_indexes.len();
        let batch_start = start.saturating_sub(n_blocks - 1);
        let first_indexes_batch: Vec<A> = first_indexes.collect_range_at(batch_start, fi_len);
        let count_indexes_all: Vec<StoredU64> = count_indexes.collect_range_at(batch_start, fi_len);
        let zero = T::from(0_usize);

        for vec in [
            &mut *min,
            &mut *max,
            &mut *median,
            &mut *pct10,
            &mut *pct25,
            &mut *pct75,
            &mut *pct90,
        ] {
            vec.truncate_if_needed_at(start)?;
        }

        // Keep the six-block population incrementally sorted. Full tuples are
        // ordered so an expired transaction can be removed exactly even when
        // several transactions have the same fee rate but different vsizes.
        let mut block_ring: VecDeque<Vec<(T, VSize)>> = VecDeque::with_capacity(n_blocks + 1);
        let mut value_cursor = source.cursor();
        let mut vsize_cursor = vsize_source.cursor();
        let mut sorted_window: Vec<(T, VSize)> = Vec::new();
        let mut merge_buf: Vec<(T, VSize)> = Vec::new();
        let mut values: Vec<T> = Vec::new();

        let mut read_block = |block_idx: usize| {
            let fi = first_indexes_batch[block_idx - batch_start].to_usize();
            let count = u64::from(count_indexes_all[block_idx - batch_start]) as usize;
            let range = effective_range(fi, count, skip_count);

            if value_cursor.position() < range.start {
                value_cursor.advance(range.start - value_cursor.position());
            }
            if vsize_cursor.position() < range.start {
                vsize_cursor.advance(range.start - vsize_cursor.position());
            }

            values.clear();
            values.reserve(range.len());
            value_cursor.for_each(range.len(), |value: T| values.push(value));
            let mut values = values.iter().copied();
            let mut block = Vec::with_capacity(range.len());
            vsize_cursor.for_each(range.len(), |vsize: VSize| {
                let value = values.next().unwrap();
                if skip_count == 0 || value > zero {
                    block.push((value, vsize));
                }
            });
            block.sort_unstable();
            block
        };

        let window_start_of_first = start.saturating_sub(n_blocks - 1);
        for block_idx in window_start_of_first..start {
            let block = read_block(block_idx);
            sorted_window.extend_from_slice(&block);
            block_ring.push_back(block);
        }
        sorted_window.sort();

        for idx in start..fi_len {
            let new_block = read_block(idx);
            merge_sorted(&mut sorted_window, &new_block, &mut merge_buf);
            block_ring.push_back(new_block);

            if block_ring.len() > n_blocks {
                let expired = block_ring.pop_front().unwrap();
                remove_sorted(&mut sorted_window, &expired, &mut merge_buf);
            }

            if sorted_window.is_empty() {
                for vec in [
                    &mut *min,
                    &mut *max,
                    &mut *median,
                    &mut *pct10,
                    &mut *pct25,
                    &mut *pct75,
                    &mut *pct90,
                ] {
                    vec.push(zero);
                }
            } else {
                max.push(sorted_window.last().unwrap().0);
                pct90.push(get_weighted_percentile(&sorted_window, 0.90));
                pct75.push(get_weighted_percentile(&sorted_window, 0.75));
                median.push(get_weighted_percentile(&sorted_window, 0.50));
                pct25.push(get_weighted_percentile(&sorted_window, 0.25));
                pct10.push(get_weighted_percentile(&sorted_window, 0.10));
                min.push(sorted_window.first().unwrap().0);
            }
        }

        let _lock = exit.lock();
        for vec in [min, max, median, pct10, pct25, pct75, pct90] {
            vec.write()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use brk_types::{VSize, get_weighted_percentile};

    use super::{effective_range, merge_sorted, remove_sorted};

    #[test]
    fn rolling_helpers_preserve_duplicate_weighted_entries() {
        let mut window = vec![(1, 100), (2, 100), (2, 200), (4, 100)];
        let block = vec![(2, 150), (3, 100)];
        let mut buffer = Vec::new();

        merge_sorted(&mut window, &block, &mut buffer);
        assert_eq!(
            window,
            vec![(1, 100), (2, 100), (2, 150), (2, 200), (3, 100), (4, 100)]
        );

        remove_sorted(&mut window, &[(2, 100), (2, 200)], &mut buffer);
        assert_eq!(window, vec![(1, 100), (2, 150), (3, 100), (4, 100)]);
    }

    #[test]
    fn effective_range_skips_each_blocks_coinbase() {
        assert_eq!(effective_range(0, 4, 1), 1..4);
        assert_eq!(effective_range(4, 3, 1), 5..7);
        assert_eq!(effective_range(7, 0, 1), 7..7);
        assert_eq!(effective_range(7, 2, 0), 7..9);
    }

    #[test]
    fn incremental_weighted_window_matches_naive_six_block_population() {
        let blocks = (0..32)
            .map(|block| {
                let mut values = vec![(0_u64, VSize::new(100))];
                values.extend((0..9).map(|tx| {
                    let rate = ((block * 7 + tx * 3) % 11) as u64;
                    let vsize = VSize::new((50 + block * 5 + tx * 13) as u64);
                    (rate, vsize)
                }));
                values
            })
            .collect::<Vec<_>>();

        let mut ring = VecDeque::new();
        let mut window = Vec::new();
        let mut buffer = Vec::new();

        for (height, raw_block) in blocks.iter().enumerate() {
            let mut block = raw_block[1..]
                .iter()
                .copied()
                .filter(|(rate, _)| *rate > 0)
                .collect::<Vec<_>>();
            block.sort_unstable();

            merge_sorted(&mut window, &block, &mut buffer);
            ring.push_back(block);
            if ring.len() > 6 {
                let expired = ring.pop_front().unwrap();
                remove_sorted(&mut window, &expired, &mut buffer);
            }

            let first = height.saturating_sub(5);
            let mut naive = blocks[first..=height]
                .iter()
                .flat_map(|block| block[1..].iter().copied())
                .filter(|(rate, _)| *rate > 0)
                .collect::<Vec<_>>();
            naive.sort_unstable();

            assert_eq!(window, naive, "wrong population at height {height}");
            for percentile in [0.10, 0.25, 0.50, 0.75, 0.90] {
                assert_eq!(
                    get_weighted_percentile(&window, percentile),
                    get_weighted_percentile(&naive, percentile),
                    "wrong percentile {percentile} at height {height}"
                );
            }
        }
    }
}
