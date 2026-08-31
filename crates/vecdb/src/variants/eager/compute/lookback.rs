use crate::{AnyVec, CheckedSub, Error, ReadableVec, StoredVec, VecIndex, VecValue, WritableVec};
use brk_exit::Exit;

use super::super::EagerVec;

impl<V> EagerVec<V>
where
    V: StoredVec,
    V::I: CheckedSub,
{
    fn compute_with_lookback<A, F>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        lookback_len: usize,
        exit: &Exit,
        transform: F,
    ) -> crate::Result<()>
    where
        A: VecValue + Default,
        F: Fn(usize, A, A) -> V::T,
    {
        self.compute_init(source.version(), max_from, exit, |this| {
            let skip = this.len();
            let end = this.batch_end(source.len());
            if skip >= end {
                return Ok(());
            }

            // Collect only the values that will be looked back to during this batch.
            // At position i, we need source[i - lookback_len] (when i >= lookback_len).
            // Reads batch_size elements instead of lookback_len.
            let prev_start = skip.saturating_sub(lookback_len);
            let prev_end = end.saturating_sub(lookback_len);
            let prev_batch = if prev_end > prev_start {
                source.collect_range_at(prev_start, prev_end)
            } else {
                vec![]
            };

            let mut prev_idx = 0;
            let mut i = skip;
            source.try_fold_range_at(skip, end, (), |(), current: A| {
                let previous = if i >= lookback_len {
                    let val = prev_batch[prev_idx].clone();
                    prev_idx += 1;
                    val
                } else {
                    A::default()
                };
                let result = transform(i, current, previous);
                this.push(result);
                i += 1;
                Ok(())
            })
        })
    }

    pub fn compute_previous_value<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        len: usize,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + Default,
        f32: From<A>,
        V::T: From<f32>,
    {
        self.compute_with_lookback(max_from, source, len, exit, |i, _, previous| {
            if i < len {
                V::T::from(f32::NAN)
            } else {
                V::T::from(f32::from(previous))
            }
        })
    }

    /// Compute N-period change. Converts source values to output type before subtraction
    /// to properly handle negative changes (e.g., unsigned source to signed output).
    pub fn compute_change<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        len: usize,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + Default + Into<V::T>,
        V::T: CheckedSub + Default,
    {
        self.compute_with_lookback(max_from, source, len, exit, |i, current, previous| {
            if i < len {
                V::T::default()
            } else {
                let current: V::T = current.into();
                let previous: V::T = previous.into();
                current.checked_sub(previous).unwrap()
            }
        })
    }

    fn compute_ratio_change_scaled<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        len: usize,
        multiplier: f32,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + Default,
        f32: From<A>,
        V::T: From<f32>,
    {
        self.compute_with_lookback(max_from, source, len, exit, |i, current, previous| {
            if i < len {
                V::T::from(f32::NAN)
            } else {
                let current_f32 = f32::from(current);
                let previous_f32 = f32::from(previous);
                V::T::from(((current_f32 / previous_f32) - 1.0) * multiplier)
            }
        })
    }

    pub fn compute_ratio_change<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        len: usize,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + Default,
        f32: From<A>,
        V::T: From<f32>,
    {
        self.compute_ratio_change_scaled(max_from, source, len, 1.0, exit)
    }

    pub fn compute_percentage_change<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        len: usize,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + Default,
        f32: From<A>,
        V::T: From<f32>,
    {
        self.compute_ratio_change_scaled(max_from, source, len, 100.0, exit)
    }

    /// Shared helper for rolling computations using variable window starts.
    pub fn compute_rolling_from_window_starts<A, F>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
        compute: F,
    ) -> crate::Result<()>
    where
        A: VecValue,
        f64: From<A>,
        V::T: From<f64>,
        F: Fn(f64, f64) -> f64,
    {
        self.compute_init(
            window_starts.version() + values.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                let starts_batch = window_starts.collect_range_at(skip, end);

                // Pre-collect values from earliest ago height to end in one shot.
                // Window starts are monotonically non-decreasing, so first is the minimum.
                let min_start = starts_batch[0].to_usize().min(skip);
                let values_data = values.collect_range_at(min_start, end);

                for (j, start) in starts_batch.into_iter().enumerate() {
                    let i = skip + j;
                    let start_usize = start.to_usize();
                    let current = f64::from(values_data[i - min_start].clone());
                    let result = if start_usize > i {
                        compute(current, f64::NAN)
                    } else if start_usize == i {
                        compute(current, current)
                    } else {
                        let previous = f64::from(values_data[start_usize - min_start].clone());
                        compute(current, previous)
                    };
                    this.push(V::T::from(result));
                }

                Ok(())
            },
        )
    }

    /// Compute ratio change using variable window starts (lookback vec).
    /// For each index i, computes `values[i] / values[window_starts[i]] - 1`.
    pub fn compute_rolling_ratio_change<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue,
        f64: From<A>,
        V::T: From<f64>,
    {
        self.compute_rolling_from_window_starts(
            max_from,
            window_starts,
            values,
            exit,
            |current, previous| {
                if previous.is_nan() || previous == 0.0 {
                    f64::NAN
                } else {
                    current / previous - 1.0
                }
            },
        )
    }

    /// Compute percentage change using variable window starts (lookback vec).
    /// For each index i, computes `(values[i] / values[window_starts[i]] - 1) * 100`.
    pub fn compute_rolling_percentage_change<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue,
        f64: From<A>,
        V::T: From<f64>,
    {
        self.compute_rolling_from_window_starts(
            max_from,
            window_starts,
            values,
            exit,
            |current, previous| {
                if previous.is_nan() || previous == 0.0 {
                    f64::NAN
                } else {
                    (current / previous - 1.0) * 100.0
                }
            },
        )
    }

    /// Compute change using variable window starts (lookback vec).
    /// For each index i, computes `values[i] - values[window_starts[i]]`.
    pub fn compute_rolling_change<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue,
        f64: From<A>,
        V::T: From<f64>,
    {
        self.compute_rolling_from_window_starts(
            max_from,
            window_starts,
            values,
            exit,
            |current, previous| {
                if previous.is_nan() {
                    0.0
                } else {
                    current - previous
                }
            },
        )
    }

    pub fn compute_cagr<A>(
        &mut self,
        max_from: V::I,
        percentage_returns: &impl ReadableVec<V::I, A>,
        days: usize,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue + Default,
        f32: From<A>,
        V::T: From<f32>,
    {
        if days == 0 || !days.is_multiple_of(365) {
            return Err(Error::InvalidArgument(
                "days must be non-zero and a multiple of 365",
            ));
        }

        let years = days / 365;

        self.compute_transform(
            max_from,
            percentage_returns,
            |(i, percentage, ..)| {
                let cagr = (((f32::from(percentage) / 100.0 + 1.0).powf(1.0 / years as f32)) - 1.0)
                    * 100.0;
                (i, V::T::from(cagr))
            },
            exit,
        )
    }

    /// For each index `i`, `output[i] = source[window_starts[i]]`.
    /// Efficiently caches lookups since window_starts are monotonically non-decreasing.
    pub fn compute_lookback<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        source: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> crate::Result<()>
    where
        A: VecValue,
        V::T: From<A>,
    {
        self.compute_init(
            window_starts.version() + source.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(source.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                let starts_batch = window_starts.collect_range_at(skip, end);

                // Pre-collect source from earliest ago height to end.
                let min_start = starts_batch[0].to_usize().min(skip);
                let source_data = source.collect_range_at(min_start, end);

                for start in starts_batch {
                    let start_usize = start.to_usize();
                    let value = source_data[start_usize - min_start].clone();
                    this.push(V::T::from(value));
                }

                Ok(())
            },
        )
    }
}
