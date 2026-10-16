use std::ops::{Add, AddAssign, Range};

use brk_exit::Exit;

use crate::{
    BinaryTransform, CheckedSub, Error, ReadableVec, Result, StoredVec, VecIndex, VecValue,
    Version, traits::writable::MAX_CACHE_SIZE,
};

/// Computes and stores derived values from source vecs, implemented for every [`StoredVec`].
///
/// Only missing values are computed (incremental updates); stale data is reset when the
/// source or computation version changes; writes are batched by the write cache size.
/// Custom computations build on [`compute_to`](Self::compute_to) (rolling statistics
/// do downstream).
pub trait ComputeVec: StoredVec {
    /// Computes up to `to` from `max_from` on, in batches as large as the write cache allows.
    /// Clamps the retained prefix to `to`, then writes after every batch, or once when there is
    /// nothing to compute (truncation only). The callback must append exactly one value for
    /// every index in its range.
    fn compute_to<F>(
        &mut self,
        max_from: Self::I,
        to: usize,
        version: Version,
        mut compute: F,
        exit: &Exit,
    ) -> Result<()>
    where
        F: FnMut(&mut Self, Range<usize>) -> Result<()>,
    {
        // Enough values per batch that `pushed_len * SIZE_OF_T >= MAX_CACHE_SIZE`, so
        // `batch_limit_reached()` fires.
        let batch_size = MAX_CACHE_SIZE.div_ceil(size_of::<Self::T>().max(1));

        {
            let _lock = exit.lock();
            self.validate_computed_version_or_reset(version)?;
            self.truncate_if_needed_at(max_from.to_usize().min(to))?;
        }

        loop {
            let from = self.len();
            let end = from.saturating_add(batch_size).min(to);
            if from < end {
                compute(self, from..end)?;
                if self.len() != end {
                    return Err(Error::InvalidArgument(
                        "compute batch callback must append one value per index",
                    ));
                }
            }

            let _lock = exit.lock();
            self.write()?;
            if end >= to {
                return Ok(());
            }
        }
    }

    fn compute_transform<A, F>(
        &mut self,
        max_from: Self::I,
        source: &impl ReadableVec<Self::I, A>,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        F: FnMut((Self::I, A, &Self)) -> (Self::I, Self::T),
    {
        self.compute_to(
            max_from,
            source.len(),
            source.version(),
            |this, range| {
                let mut i = range.start;
                source.fold_range_at(range.start, range.end, (), |(), b: A| {
                    let (idx, v) = t((Self::I::from(i), b, &*this));
                    i += 1;
                    this.debug_checked_push(idx, v);
                });
                Ok(())
            },
            exit,
        )
    }

    fn compute_transform2<A, B, F>(
        &mut self,
        max_from: Self::I,
        other1: &impl ReadableVec<Self::I, A>,
        other2: &impl ReadableVec<Self::I, B>,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        F: FnMut((Self::I, A, B, &Self)) -> (Self::I, Self::T),
    {
        self.compute_to(
            max_from,
            other1.len().min(other2.len()),
            other1.version() + other2.version(),
            |this, range| {
                let batch2 = other2.collect_range_at(range.start, range.end);
                let mut iter2 = batch2.into_iter();
                let mut i = range.start;

                other1.fold_range_at(range.start, range.end, (), |(), b: A| {
                    let (idx, v) = t((Self::I::from(i), b, iter2.next().unwrap(), &*this));
                    i += 1;
                    this.debug_checked_push(idx, v);
                });
                Ok(())
            },
            exit,
        )
    }

    fn compute_binary<A, B, F>(
        &mut self,
        max_from: Self::I,
        source1: &impl ReadableVec<Self::I, A>,
        source2: &impl ReadableVec<Self::I, B>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        F: BinaryTransform<A, B, Self::T>,
    {
        self.compute_transform2(
            max_from,
            source1,
            source2,
            |(h, a, b, ..)| (h, F::apply(a, b)),
            exit,
        )
    }

    fn compute_transform3<A, B, C, F>(
        &mut self,
        max_from: Self::I,
        other1: &impl ReadableVec<Self::I, A>,
        other2: &impl ReadableVec<Self::I, B>,
        other3: &impl ReadableVec<Self::I, C>,
        mut t: F,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        C: VecValue,
        F: FnMut((Self::I, A, B, C, &Self)) -> (Self::I, Self::T),
    {
        self.compute_to(
            max_from,
            other1.len().min(other2.len()).min(other3.len()),
            other1.version() + other2.version() + other3.version(),
            |this, range| {
                let batch2 = other2.collect_range_at(range.start, range.end);
                let batch3 = other3.collect_range_at(range.start, range.end);
                let mut iter2 = batch2.into_iter();
                let mut iter3 = batch3.into_iter();
                let mut i = range.start;

                other1.fold_range_at(range.start, range.end, (), |(), b: A| {
                    let (idx, v) = t((
                        Self::I::from(i),
                        b,
                        iter2.next().unwrap(),
                        iter3.next().unwrap(),
                        &*this,
                    ));
                    i += 1;
                    this.debug_checked_push(idx, v);
                });
                Ok(())
            },
            exit,
        )
    }

    fn compute_subtract(
        &mut self,
        max_from: Self::I,
        subtracted: &impl ReadableVec<Self::I, Self::T>,
        subtracter: &impl ReadableVec<Self::I, Self::T>,
        exit: &Exit,
    ) -> Result<()>
    where
        Self::T: CheckedSub,
    {
        self.compute_transform2(
            max_from,
            subtracted,
            subtracter,
            |(i, v1, v2, ..)| {
                (
                    i,
                    v1.checked_sub(v2)
                        .expect("subtraction underflow in compute_subtract"),
                )
            },
            exit,
        )
    }

    /// Compute cumulative sum from a source vec.
    ///
    /// Each value in the result is the sum of all values from the source up to
    /// and including that index.
    fn compute_cumulative<S>(
        &mut self,
        max_from: Self::I,
        source: &impl ReadableVec<Self::I, S>,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue + Into<Self::T>,
        Self::T: Default + AddAssign + Copy,
    {
        self.compute_to(
            max_from,
            source.len(),
            source.version(),
            |this, range| {
                let mut cumulative_val = this.collect_last().unwrap_or_default();
                source.try_fold_range_at(range.start, range.end, (), |(), v: S| {
                    cumulative_val += v.into();
                    this.push(cumulative_val);
                    Ok(())
                })
            },
            exit,
        )
    }

    /// Compute cumulative sum from a custom binary transform of two source vecs.
    ///
    /// Each value in the result is the cumulative sum of `transform(source1[i], source2[i])`
    /// for all indices up to and including i.
    fn compute_cumulative_transformed_binary<S1, S2, F>(
        &mut self,
        max_from: Self::I,
        source1: &impl ReadableVec<Self::I, S1>,
        source2: &impl ReadableVec<Self::I, S2>,
        mut transform: F,
        exit: &Exit,
    ) -> Result<()>
    where
        S1: VecValue,
        S2: VecValue,
        Self::T: Default + AddAssign + Copy,
        F: FnMut(S1, S2) -> Self::T,
    {
        self.compute_to(
            max_from,
            source1.len().min(source2.len()),
            source1.version() + source2.version(),
            |this, range| {
                let mut cumulative_val = this.collect_last().unwrap_or_default();
                let batch2 = source2.collect_range_at(range.start, range.end);
                let mut iter2 = batch2.into_iter();

                source1.try_fold_range_at(range.start, range.end, (), |(), v1: S1| {
                    let v2 = iter2.next().unwrap();
                    cumulative_val += transform(v1, v2);
                    this.push(cumulative_val);
                    Ok(())
                })
            },
            exit,
        )
    }

    /// Transform and sum each contiguous group, appending running totals.
    fn compute_cumulative_sum_from_indexes<A, B, S>(
        &mut self,
        max_from: Self::I,
        first_indexes: &impl ReadableVec<Self::I, A>,
        indexes_count: &impl ReadableVec<Self::I, B>,
        source: &impl ReadableVec<A, S>,
        mut transform: impl FnMut(S) -> Self::T,
        exit: &Exit,
    ) -> Result<()>
    where
        Self::T: Default + Copy + Add<Output = Self::T> + AddAssign,
        A: VecIndex + VecValue,
        B: VecValue,
        S: VecValue,
        usize: From<B>,
    {
        let mut cumulative = None;
        let mut emit = |this: &mut Self, sum: Option<Self::T>| {
            let cumulative =
                cumulative.get_or_insert_with(|| this.collect_last().unwrap_or_default());
            // Empty groups preserve the total without an extra floating-point addition.
            if let Some(sum) = sum {
                *cumulative += sum;
            }
            this.push(*cumulative);
        };
        self.compute_to(
            max_from,
            indexes_count.len().min(first_indexes.len()),
            first_indexes.version() + indexes_count.version() + source.version(),
            |this, range| {
                let pos = first_indexes
                    .collect_one_at(range.start)
                    .unwrap()
                    .to_usize();

                let counts_batch: Vec<usize> = indexes_count
                    .collect_range_at(range.start, range.end)
                    .into_iter()
                    .map(usize::from)
                    .collect();
                let total_count: usize = counts_batch.iter().sum();

                let mut group_idx = 0usize;

                // Skip leading zero-count groups
                while group_idx < counts_batch.len() && counts_batch[group_idx] == 0 {
                    emit(this, None);
                    group_idx += 1;
                }

                if group_idx < counts_batch.len() {
                    let mut remaining = counts_batch[group_idx];

                    source.fold_range_at(
                        pos,
                        pos + total_count,
                        Self::T::default(),
                        |sum, value| {
                            let sum = sum + transform(value);
                            remaining -= 1;
                            if remaining == 0 {
                                emit(this, Some(sum));
                                group_idx += 1;
                                while group_idx < counts_batch.len() && counts_batch[group_idx] == 0
                                {
                                    emit(this, None);
                                    group_idx += 1;
                                }
                                if group_idx < counts_batch.len() {
                                    remaining = counts_batch[group_idx];
                                }
                                Self::T::default()
                            } else {
                                sum
                            }
                        },
                    );
                }

                Ok(())
            },
            exit,
        )
    }
}

impl<V: StoredVec> ComputeVec for V {}
