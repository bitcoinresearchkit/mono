use std::collections::VecDeque;

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, StoredU64, VSize, get_percentile, get_weighted_percentile};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, CheckedSub, Database, Exit, ReadableVec, Rw, StorageMode, VecIndex,
    VecValue, Version, WritableVec,
};

use crate::{
    indexes,
    internal::{ComputedVecValue, DistributionStats, NumericValue, PerBlock},
};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct PerBlockDistribution<T: ComputedVecValue + PartialOrd + JsonSchema, M: StorageMode = Rw>(
    pub DistributionStats<PerBlock<T, M>>,
);

impl<T: NumericValue + JsonSchema> PerBlockDistribution<T> {
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|suffix| {
            PerBlock::forced_import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }

    pub(crate) fn compute_with_skip<A>(
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
    pub(crate) fn compute_with_skip_weighted<A>(
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

    pub(crate) fn compute_from_nblocks<A>(
        &mut self,
        max_from: Height,
        source: &(impl ReadableVec<A, T> + Sized),
        first_indexes: &impl ReadableVec<Height, A>,
        count_indexes: &impl ReadableVec<Height, StoredU64>,
        n_blocks: usize,
        exit: &Exit,
    ) -> Result<()>
    where
        T: CheckedSub,
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
            if cursor.position() < fi {
                cursor.advance(fi - cursor.position());
            }
            let mut bv = Vec::with_capacity(count);
            cursor.for_each(count, |v: T| bv.push(v));
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
            if cursor.position() < fi {
                cursor.advance(fi - cursor.position());
            }
            let mut new_block = Vec::with_capacity(count);
            cursor.for_each(count, |v: T| new_block.push(v));
            new_block.sort_unstable();

            // Merge-insert new sorted block into sorted_window: O(n+m)
            merge_buf.clear();
            merge_buf.reserve(sorted_window.len() + new_block.len());
            let (mut si, mut ni) = (0, 0);
            while si < sorted_window.len() && ni < new_block.len() {
                if sorted_window[si] <= new_block[ni] {
                    merge_buf.push(sorted_window[si]);
                    si += 1;
                } else {
                    merge_buf.push(new_block[ni]);
                    ni += 1;
                }
            }
            merge_buf.extend_from_slice(&sorted_window[si..]);
            merge_buf.extend_from_slice(&new_block[ni..]);
            std::mem::swap(&mut sorted_window, &mut merge_buf);

            block_ring.push_back(new_block);

            // Expire oldest block: merge-filter its sorted values from sorted_window in O(n)
            if block_ring.len() > n_blocks {
                let expired = block_ring.pop_front().unwrap();

                merge_buf.clear();
                merge_buf.reserve(sorted_window.len());
                let mut ei = 0;
                for &v in &sorted_window {
                    if ei < expired.len() && v == expired[ei] {
                        ei += 1;
                    } else {
                        merge_buf.push(v);
                    }
                }
                std::mem::swap(&mut sorted_window, &mut merge_buf);
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
}
