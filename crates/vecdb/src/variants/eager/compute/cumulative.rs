use std::ops::{Add, AddAssign};

use crate::{AnyVec, Exit, ReadableVec, StoredVec, VecIndex, VecValue, WritableVec};

use super::super::EagerVec;

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Compute cumulative sum from a source vec.
    ///
    /// Each value in the result is the sum of all values from the source up to
    /// and including that index.
    pub fn compute_cumulative<S>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, S>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        S: VecValue + Into<V::T>,
        V::T: Default + AddAssign + Copy,
    {
        self.compute_init(source.version(), max_from, exit, |this| {
            let skip = this.len();
            let end = this.batch_end(source.len());
            if skip >= end {
                return Ok(());
            }

            let mut cumulative_val = if skip > 0 {
                this.collect_one_at(skip - 1).unwrap()
            } else {
                V::T::default()
            };

            let mut i = skip;
            source.try_fold_range_at(skip, end, (), |(), v: S| {
                cumulative_val += v.into();
                this.push(cumulative_val);
                i += 1;
                Ok(())
            })
        })
    }

    /// Compute cumulative sum from adding two source vecs element-wise.
    ///
    /// Each value in the result is the cumulative sum of `source1[i] + source2[i]`
    /// for all indices up to and including i.
    pub fn compute_cumulative_binary<S1, S2>(
        &mut self,
        max_from: V::I,
        source1: &impl ReadableVec<V::I, S1>,
        source2: &impl ReadableVec<V::I, S2>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        S1: VecValue + Into<V::T>,
        S2: VecValue + Into<V::T>,
        V::T: Default + AddAssign + Add<Output = V::T> + Copy,
    {
        self.compute_cumulative_transformed_binary(
            max_from,
            source1,
            source2,
            |v1: S1, v2: S2| v1.into() + v2.into(),
            exit,
        )
    }

    /// Compute cumulative sum from a custom binary transform of two source vecs.
    ///
    /// Each value in the result is the cumulative sum of `transform(source1[i], source2[i])`
    /// for all indices up to and including i.
    pub fn compute_cumulative_transformed_binary<S1, S2, F>(
        &mut self,
        max_from: V::I,
        source1: &impl ReadableVec<V::I, S1>,
        source2: &impl ReadableVec<V::I, S2>,
        mut transform: F,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        S1: VecValue,
        S2: VecValue,
        V::T: Default + AddAssign + Copy,
        F: FnMut(S1, S2) -> V::T,
    {
        let target_len = source1.len().min(source2.len());

        self.compute_init(
            source1.version() + source2.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let end = this.batch_end(target_len);
                if skip >= end {
                    return Ok(());
                }

                let mut cumulative_val = if skip > 0 {
                    this.collect_one_at(skip - 1).unwrap()
                } else {
                    V::T::default()
                };

                let batch2 = source2.collect_range_at(skip, end);
                let mut iter2 = batch2.into_iter();
                let mut i = skip;

                source1.try_fold_range_at(skip, end, (), |(), v1: S1| {
                    let v2 = iter2.next().unwrap();
                    cumulative_val += transform(v1, v2);
                    this.push(cumulative_val);
                    i += 1;
                    Ok(())
                })
            },
        )
    }

    /// Compute cumulative count of values matching a predicate.
    ///
    /// Each value in the result is the count of values from the source up to
    /// and including that index where the predicate returns true.
    pub fn compute_cumulative_count<S, P>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, S>,
        predicate: P,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        S: VecValue,
        V::T: From<usize> + AddAssign + Copy,
        P: Fn(&S) -> bool,
    {
        self.compute_cumulative_count_from(max_from, source, V::I::from(0), predicate, exit)
    }

    /// Compute rolling count of values matching a predicate within a window.
    pub fn compute_rolling_count<S, P>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, S>,
        window_size: usize,
        predicate: P,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        S: VecValue,
        V::T: From<usize> + Into<usize> + Copy,
        P: Fn(&S) -> bool,
    {
        self.compute_init(source.version(), max_from, exit, |this| {
            let skip = this.len();
            let end = this.batch_end(source.len());
            if skip >= end {
                return Ok(());
            }

            // Recover count from stored output instead of rebuilding full window.
            // Reads 1 element from self instead of window_size from source.
            let mut count: usize = if skip > 0 {
                this.collect_one_at(skip - 1).unwrap().into()
            } else {
                0
            };

            // Collect only the elements that leave the window during this batch.
            // At position i, source[i - window_size] leaves (when i >= window_size).
            // Reads batch_size elements instead of window_size.
            let leave_start = skip.saturating_sub(window_size);
            let leave_end = end.saturating_sub(window_size);
            let leave_batch = if leave_end > leave_start {
                source.collect_range_at(leave_start, leave_end)
            } else {
                vec![]
            };

            let mut leave_idx = 0;
            let mut i = skip;
            source.try_fold_range_at(skip, end, (), |(), v: S| {
                if i >= window_size {
                    if predicate(&leave_batch[leave_idx]) {
                        count -= 1;
                    }
                    leave_idx += 1;
                }
                if predicate(&v) {
                    count += 1;
                }

                this.push(V::T::from(count));
                i += 1;
                Ok(())
            })
        })
    }

    /// Compute cumulative count of values matching a predicate, starting from a specific index.
    ///
    /// Values before `from` will be 0. Starting at `from`, counts values where predicate is true.
    pub fn compute_cumulative_count_from<S, P>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, S>,
        from: V::I,
        predicate: P,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        S: VecValue,
        V::T: From<usize> + AddAssign + Copy,
        P: Fn(&S) -> bool,
    {
        let from_usize = from.to_usize();
        let mut count: Option<V::T> = None;
        self.compute_transform(
            max_from,
            source,
            |(i, v, this)| {
                let idx = i.to_usize();
                if count.is_none() {
                    count = Some(if idx > 0 {
                        this.collect_one_at(idx - 1).unwrap()
                    } else {
                        V::T::from(0_usize)
                    });
                }
                if idx >= from_usize && predicate(&v) {
                    *count.as_mut().unwrap() += V::T::from(1_usize);
                }
                (i, count.unwrap())
            },
            exit,
        )
    }
}
