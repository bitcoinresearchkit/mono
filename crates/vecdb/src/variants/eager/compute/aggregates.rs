use std::ops::{Add, AddAssign};

use brk_exit::Exit;

use super::super::EagerVec;
use crate::{
    AnyVec, ReadableVec, Result, SaturatingAdd, StoredVec, VecIndex, VecValue, WritableVec,
};

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Sum each contiguous group, saturating on overflow.
    pub fn compute_sum_from_indexes<A, B>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        indexes_count: &impl ReadableVec<V::I, B>,
        source: &impl ReadableVec<A, V::T>,
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: Default + SaturatingAdd,
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        self.compute_grouped_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            SaturatingAdd::saturating_add,
            |this, sum| this.push(sum.unwrap_or_default()),
            exit,
        )
    }

    /// Transform and sum each contiguous group, appending running totals.
    pub fn compute_cumulative_sum_from_indexes<A, B, S>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        indexes_count: &impl ReadableVec<V::I, B>,
        source: &impl ReadableVec<A, S>,
        mut transform: impl FnMut(S) -> V::T,
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: Default + Copy + Add<Output = V::T> + AddAssign,
        A: VecIndex + VecValue,
        B: VecValue,
        S: VecValue,
        usize: From<B>,
    {
        let mut cumulative = None;
        self.compute_grouped_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            |sum, value| sum + transform(value),
            |this, sum| {
                let cumulative =
                    cumulative.get_or_insert_with(|| this.collect_last().unwrap_or_default());
                // Empty groups preserve the total without an extra floating-point addition.
                if let Some(sum) = sum {
                    *cumulative += sum;
                }
                this.push(*cumulative);
            },
            exit,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_grouped_from_indexes<A, B, S>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        indexes_count: &impl ReadableVec<V::I, B>,
        source: &impl ReadableVec<A, S>,
        mut reduce: impl FnMut(V::T, S) -> V::T,
        mut emit: impl FnMut(&mut Self, Option<V::T>),
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: Default,
        A: VecIndex + VecValue,
        B: VecValue,
        S: VecValue,
        usize: From<B>,
    {
        self.compute_init(
            first_indexes.version() + indexes_count.version() + source.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = indexes_count.len().min(first_indexes.len());
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let pos = first_indexes.collect_one_at(skip).unwrap().to_usize();

                let counts_batch: Vec<usize> = indexes_count
                    .collect_range_at(skip, end)
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

                    source.fold_range_at(pos, pos + total_count, V::T::default(), |sum, value| {
                        let sum = reduce(sum, value);
                        remaining -= 1;
                        if remaining == 0 {
                            emit(this, Some(sum));
                            group_idx += 1;
                            while group_idx < counts_batch.len() && counts_batch[group_idx] == 0 {
                                emit(this, None);
                                group_idx += 1;
                            }
                            if group_idx < counts_batch.len() {
                                remaining = counts_batch[group_idx];
                            }
                            V::T::default()
                        } else {
                            sum
                        }
                    });
                }

                Ok(())
            },
        )
    }
}
