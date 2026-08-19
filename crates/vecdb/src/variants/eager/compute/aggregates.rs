use std::ops::Add;

use crate::{
    AnyVec, CheckedSub, Error, Exit, ReadableVec, SaturatingAdd, StoredVec, VecIndex, VecValue,
    Version, WritableVec, unlikely,
};

use super::super::EagerVec;

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    fn compute_aggregate_of_others<O, F>(
        &mut self,
        max_from: V::I,
        others: &[&O],
        exit: &Exit,
        aggregate: F,
    ) -> crate::Result<()>
    where
        O: ReadableVec<V::I, V::T>,
        F: Fn(&mut V::T, V::T),
    {
        if others.is_empty() {
            return Err(Error::InvalidArgument(
                "others must have at least one element",
            ));
        }

        self.compute_init(
            others.iter().map(|v| v.version()).sum(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = others.iter().map(|v| v.len()).min().unwrap();
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let mut batches = others.iter().map(|v| v.collect_range_at(skip, end));
                let mut aggregated = batches.next().unwrap();

                for batch in batches {
                    debug_assert_eq!(aggregated.len(), batch.len());
                    for (value, other) in aggregated.iter_mut().zip(batch) {
                        aggregate(value, other);
                    }
                }

                for value in aggregated {
                    this.push(value);
                }

                Ok(())
            },
        )
    }

    pub fn compute_sum_of_others<O>(
        &mut self,
        max_from: V::I,
        others: &[&O],
        exit: &Exit,
    ) -> crate::Result<()>
    where
        O: ReadableVec<V::I, V::T>,
        V::T: Add<V::T, Output = V::T>,
    {
        self.compute_aggregate_of_others(max_from, others, exit, |sum, value| {
            *sum = sum.clone() + value;
        })
    }

    pub fn compute_min_of_others<O>(
        &mut self,
        max_from: V::I,
        others: &[&O],
        exit: &Exit,
    ) -> crate::Result<()>
    where
        O: ReadableVec<V::I, V::T>,
        V::T: Add<V::T, Output = V::T> + Ord,
    {
        self.compute_aggregate_of_others(max_from, others, exit, |min, value| {
            if value.lt(min) {
                *min = value;
            }
        })
    }

    pub fn compute_max_of_others<O>(
        &mut self,
        max_from: V::I,
        others: &[&O],
        exit: &Exit,
    ) -> crate::Result<()>
    where
        O: ReadableVec<V::I, V::T>,
        V::T: Add<V::T, Output = V::T> + Ord,
    {
        self.compute_aggregate_of_others(max_from, others, exit, |max, value| {
            if value.gt(max) {
                *max = value;
            }
        })
    }

    /// Computes weighted average: sum(weight_i * value_i) / sum(weight_i)
    ///
    /// Takes parallel slices of weight and value vecs from multiple sources.
    /// For each index, computes the weighted average across all sources.
    /// Returns zero if total weight is zero.
    pub fn compute_weighted_average_of_others<W, OW, OV>(
        &mut self,
        max_from: V::I,
        weights: &[&OW],
        values: &[&OV],
        exit: &Exit,
    ) -> crate::Result<()>
    where
        W: VecValue + Into<f64>,
        OW: ReadableVec<V::I, W>,
        OV: ReadableVec<V::I, V::T>,
        V::T: Into<f64> + From<f64>,
    {
        if weights.len() != values.len() {
            return Err(Error::InvalidArgument(
                "weights and values must have same length",
            ));
        }

        if weights.is_empty() {
            return Err(Error::InvalidArgument(
                "weights and values must have at least one element",
            ));
        }

        self.compute_init(
            weights.iter().map(|v| v.version()).sum::<Version>()
                + values.iter().map(|v| v.version()).sum(),
            max_from,
            exit,
            |this| {
                let skip = this.len();

                let source_end = weights
                    .iter()
                    .map(|w| w.len())
                    .chain(values.iter().map(|v| v.len()))
                    .min()
                    .unwrap_or(0);
                let end = this.batch_end(source_end);

                if skip >= end {
                    return Ok(());
                }

                let weight_batches: Vec<Vec<W>> = weights
                    .iter()
                    .map(|w| w.collect_range_at(skip, end))
                    .collect();
                let value_batches: Vec<Vec<V::T>> = values
                    .iter()
                    .map(|v| v.collect_range_at(skip, end))
                    .collect();

                for j in 0..(end - skip) {
                    let mut total_weight = 0.0_f64;
                    let mut weighted_sum = 0.0_f64;

                    for (w_batch, v_batch) in weight_batches.iter().zip(value_batches.iter()) {
                        let weight: f64 = w_batch[j].clone().into();
                        let value: f64 = v_batch[j].clone().into();

                        if weight > 0.0 {
                            total_weight += weight;
                            weighted_sum += weight * value;
                        }
                    }

                    let result = if total_weight > 0.0 {
                        V::T::from(weighted_sum / total_weight)
                    } else {
                        V::T::from(0.0)
                    };

                    this.push(result);
                }

                Ok(())
            },
        )
    }

    pub fn compute_sum_from_indexes<A, B>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        indexes_count: &impl ReadableVec<V::I, B>,
        source: &(impl ReadableVec<A, V::T> + Sized),
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::T: Default + SaturatingAdd,
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        self.compute_init(
            first_indexes.version() + indexes_count.version() + source.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = indexes_count.len();
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let pos = if skip < first_indexes.len() {
                    first_indexes.collect_one_at(skip).unwrap().to_usize()
                } else {
                    return Ok(());
                };

                let counts_batch: Vec<usize> = indexes_count
                    .collect_range_at(skip, end)
                    .into_iter()
                    .map(usize::from)
                    .collect();
                let total_count: usize = counts_batch.iter().sum();

                let mut group_idx = 0usize;

                // Skip leading zero-count groups
                while group_idx < counts_batch.len() && counts_batch[group_idx] == 0 {
                    this.push(V::T::default());
                    group_idx += 1;
                }

                if group_idx < counts_batch.len() {
                    let mut remaining = counts_batch[group_idx];

                    source.fold_range_at(
                        pos,
                        pos + total_count,
                        V::T::default(),
                        |sum, val: V::T| {
                            let sum = sum.saturating_add(val);
                            remaining -= 1;
                            if unlikely(remaining == 0) {
                                this.push(sum);
                                group_idx += 1;
                                while group_idx < counts_batch.len() && counts_batch[group_idx] == 0
                                {
                                    this.push(V::T::default());
                                    group_idx += 1;
                                }
                                if group_idx < counts_batch.len() {
                                    remaining = counts_batch[group_idx];
                                }
                                V::T::default()
                            } else {
                                sum
                            }
                        },
                    );
                }

                Ok(())
            },
        )
    }

    pub fn compute_filtered_sum_from_indexes<A, B>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        indexes_count: &impl ReadableVec<V::I, B>,
        source: &(impl ReadableVec<A, V::T> + Sized),
        mut filter: impl FnMut(&V::T) -> bool,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::T: Default + SaturatingAdd,
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        self.compute_init(
            first_indexes.version() + indexes_count.version() + source.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = indexes_count.len();
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let pos = if skip < first_indexes.len() {
                    first_indexes.collect_one_at(skip).unwrap().to_usize()
                } else {
                    return Ok(());
                };

                let counts_batch: Vec<usize> = indexes_count
                    .collect_range_at(skip, end)
                    .into_iter()
                    .map(usize::from)
                    .collect();
                let total_count: usize = counts_batch.iter().sum();

                let mut group_idx = 0usize;

                while group_idx < counts_batch.len() && counts_batch[group_idx] == 0 {
                    this.push(V::T::default());
                    group_idx += 1;
                }

                if group_idx < counts_batch.len() {
                    let mut remaining = counts_batch[group_idx];

                    source.fold_range_at(
                        pos,
                        pos + total_count,
                        V::T::default(),
                        |sum, val: V::T| {
                            let sum = if filter(&val) {
                                sum.saturating_add(val)
                            } else {
                                sum
                            };
                            remaining -= 1;
                            if unlikely(remaining == 0) {
                                this.push(sum);
                                group_idx += 1;
                                while group_idx < counts_batch.len() && counts_batch[group_idx] == 0
                                {
                                    this.push(V::T::default());
                                    group_idx += 1;
                                }
                                if group_idx < counts_batch.len() {
                                    remaining = counts_batch[group_idx];
                                }
                                V::T::default()
                            } else {
                                sum
                            }
                        },
                    );
                }

                Ok(())
            },
        )
    }

    pub fn compute_count_from_indexes<A, B>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        other_to_else: &impl ReadableVec<A, B>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::T: From<A>,
        A: VecValue
            + VecIndex
            + Copy
            + Add<usize, Output = A>
            + CheckedSub<A>
            + TryInto<V::T>
            + Default,
        <A as TryInto<V::T>>::Error: core::error::Error + 'static,
        B: VecValue,
    {
        self.compute_count_from_indexes_with(
            max_from,
            first_indexes,
            other_to_else,
            |first, next_first| next_first - first,
            exit,
        )
    }

    pub fn compute_filtered_count_from_indexes<A, B>(
        &mut self,
        max_from: V::I,
        first_indexes: &impl ReadableVec<V::I, A>,
        other_to_else: &impl ReadableVec<A, B>,
        mut filter: impl FnMut(A) -> bool,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::T: From<A>,
        A: VecValue
            + VecIndex
            + Copy
            + Add<usize, Output = A>
            + CheckedSub<A>
            + TryInto<V::T>
            + Default,
        B: VecValue,
        <A as TryInto<V::T>>::Error: core::error::Error + 'static,
    {
        self.compute_count_from_indexes_with(
            max_from,
            first_indexes,
            other_to_else,
            |first, next_first| (first..next_first).filter(|i| filter(A::from(*i))).count(),
            exit,
        )
    }

    fn compute_count_from_indexes_with<A, B>(
        &mut self,
        max_from: V::I,
        first_indexes: &(impl ReadableVec<V::I, A> + Sized),
        other_to_else: &impl ReadableVec<A, B>,
        mut count_fn: impl FnMut(usize, usize) -> usize,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        V::T: From<A>,
        A: VecValue
            + VecIndex
            + Copy
            + Add<usize, Output = A>
            + CheckedSub<A>
            + TryInto<V::T>
            + Default,
        B: VecValue,
        <A as TryInto<V::T>>::Error: core::error::Error + 'static,
    {
        self.compute_init(
            first_indexes.version() + other_to_else.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_end = first_indexes.len();
                let end = this.batch_end(source_end);
                if skip >= end {
                    return Ok(());
                }

                let prev_val = first_indexes.collect_one_at(skip).unwrap().to_usize();

                let last_prev =
                    first_indexes.fold_range_at(skip + 1, end, prev_val, |prev, fi: A| {
                        let next = fi.to_usize();
                        this.push(V::T::from(A::from(count_fn(prev, next))));
                        next
                    });

                let next_first = if end < first_indexes.len() {
                    first_indexes.collect_one_at(end).unwrap().to_usize()
                } else {
                    other_to_else.len()
                };
                this.push(V::T::from(A::from(count_fn(last_prev, next_first))));

                Ok(())
            },
        )
    }
}
