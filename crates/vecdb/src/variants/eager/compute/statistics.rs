use std::ops::{Add, AddAssign, SubAssign};

use brk_exit::Exit;

use super::super::EagerVec;
use crate::{AnyVec, ReadableVec, Result, StoredVec, VecIndex, VecValue, Version, WritableVec};

impl<V> EagerVec<V>
where
    V: StoredVec,
{
    /// Compute rolling sum with variable window starts.
    /// For each index i, computes sum of values from `window_starts[i]` to i (inclusive).
    pub fn compute_rolling_sum<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        V::T: From<A> + Default + AddAssign + SubAssign,
    {
        // Cursor for the leaving-value reads — persists across batches so each
        // compressed page is decompressed at most once instead of once per element.
        let mut leaving = values.cursor();

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

                let (mut running_sum, mut prev_start) = if skip > 0 {
                    let prev_idx = skip - 1;
                    let prev_start = window_starts.collect_one_at(prev_idx).unwrap();
                    let sum = this.collect_one_at(prev_idx).unwrap();
                    // Position cursor at the current window start.
                    if leaving.position() < prev_start.to_usize() {
                        leaving.advance(prev_start.to_usize() - leaving.position());
                    }
                    (sum, prev_start)
                } else {
                    (V::T::default(), V::I::from(0))
                };

                let starts_batch = window_starts.collect_range_at(skip, end);
                let values_batch = values.collect_range_at(skip, end);

                for (start, value) in starts_batch.into_iter().zip(values_batch) {
                    running_sum += V::T::from(value);

                    if prev_start < start {
                        let n = start.to_usize() - prev_start.to_usize();
                        leaving.for_each(n, |v: A| {
                            running_sum -= V::T::from(v);
                        });
                        prev_start = start;
                    }

                    this.push(running_sum.clone());
                }

                Ok(())
            },
        )
    }

    /// Compute rolling average with variable window starts.
    /// For each index i, computes mean of values from `window_starts[i]` to i (inclusive).
    pub fn compute_rolling_average<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<V::T>,
        V::T: From<f64> + Default,
    {
        // Cursor for the leaving-value reads — persists across batches so each
        // compressed page is decompressed at most once instead of once per element.
        let mut leaving = values.cursor();

        self.compute_init(
            window_starts.version() + values.version() + Version::new(2),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                // Recover running_sum from stored average (fast but lossy).
                let (mut running_sum, mut prev_start) = if skip > 0 {
                    let prev_idx = skip - 1;
                    let prev_start = window_starts.collect_one_at(prev_idx).unwrap();
                    if leaving.position() < prev_start.to_usize() {
                        leaving.advance(prev_start.to_usize() - leaving.position());
                    }
                    let stored_avg = f64::from(this.collect_one_at(prev_idx).unwrap());
                    let window_count = prev_idx + 1 - prev_start.to_usize();
                    (stored_avg * window_count as f64, prev_start)
                } else {
                    (0.0_f64, V::I::from(0))
                };

                let starts_batch = window_starts.collect_range_at(skip, end);
                let values_batch = values.collect_range_at(skip, end);

                for (j, (start, value)) in starts_batch.into_iter().zip(values_batch).enumerate() {
                    let i = skip + j;
                    running_sum += f64::from(value);

                    if prev_start < start {
                        let n = start.to_usize() - prev_start.to_usize();
                        leaving.for_each(n, |v: A| {
                            running_sum -= f64::from(v);
                        });
                        prev_start = start;
                    }

                    let count = i - start.to_usize() + 1;
                    let avg = running_sum / count as f64;
                    this.push(V::T::from(avg));
                }

                Ok(())
            },
        )
    }

    /// Compute rolling standard deviation with variable window starts.
    /// For each index `i`, computes SD of values from `window_starts[i]` to `i` (inclusive),
    /// using the provided rolling mean.
    /// `SD = sqrt(E[X²] - E[X]²)` where `E[X²]` is the rolling mean of squares
    /// and `E[X]` is the rolling mean from the `mean` parameter.
    pub fn compute_rolling_sd<A, B>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        mean: &impl ReadableVec<V::I, B>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        f64: From<A> + From<B> + From<V::T>,
        V::T: From<f64>,
    {
        let mut leaving = values.cursor();

        self.compute_init(
            window_starts.version() + values.version() + mean.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len()).min(mean.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                let (mut running_sum_sq, mut prev_start) = if skip > 0 {
                    let prev_idx = skip - 1;
                    let prev_start = window_starts.collect_one_at(prev_idx).unwrap();
                    let count = (prev_idx + 1 - prev_start.to_usize()) as f64;
                    let sd_val = f64::from(this.collect_one_at(prev_idx).unwrap());
                    let mean_val = f64::from(mean.collect_one_at(prev_idx).unwrap());
                    let sum_sq = (sd_val * sd_val + mean_val * mean_val) * count;
                    if leaving.position() < prev_start.to_usize() {
                        leaving.advance(prev_start.to_usize() - leaving.position());
                    }
                    (sum_sq, prev_start)
                } else {
                    (0.0f64, V::I::from(0))
                };

                let starts_batch = window_starts.collect_range_at(skip, end);
                let values_batch = values.collect_range_at(skip, end);
                let mean_batch = mean.collect_range_at(skip, end);

                for (j, ((start, value), m)) in starts_batch
                    .into_iter()
                    .zip(values_batch)
                    .zip(mean_batch)
                    .enumerate()
                {
                    let i = skip + j;
                    let val = f64::from(value);
                    running_sum_sq += val * val;

                    if prev_start < start {
                        let n = start.to_usize() - prev_start.to_usize();
                        leaving.for_each(n, |v: A| {
                            let old = f64::from(v);
                            running_sum_sq -= old * old;
                        });
                        prev_start = start;
                    }

                    let count = (i - start.to_usize() + 1) as f64;
                    let mean_val = f64::from(m);
                    let variance = (running_sum_sq / count - mean_val * mean_val).max(0.0);
                    this.push(V::T::from(variance.sqrt()));
                }

                Ok(())
            },
        )
    }

    /// Compute expanding (all-time) standard deviation.
    /// For each index `i`, computes SD of all values from 0 to `i` (inclusive).
    /// `SD = sqrt(E[X²] - E[X]²)`.
    pub fn compute_expanding_sd<A, B>(
        &mut self,
        max_from: V::I,
        values: &impl ReadableVec<V::I, A>,
        mean: &impl ReadableVec<V::I, B>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        f64: From<A> + From<B> + From<V::T>,
        V::T: From<f64>,
    {
        self.compute_init(values.version() + mean.version(), max_from, exit, |this| {
            let skip = this.len();
            let source_len = values.len().min(mean.len());
            let end = this.batch_end(source_len);
            if skip >= end {
                return Ok(());
            }

            let mut running_sum_sq = if skip > 0 {
                let count = skip as f64;
                let sd_val = f64::from(this.collect_one_at(skip - 1).unwrap());
                let mean_val = f64::from(mean.collect_one_at(skip - 1).unwrap());
                (sd_val * sd_val + mean_val * mean_val) * count
            } else {
                0.0f64
            };

            let values_batch = values.collect_range_at(skip, end);
            let mean_batch = mean.collect_range_at(skip, end);

            for (j, (value, m)) in values_batch.into_iter().zip(mean_batch).enumerate() {
                let i = skip + j;
                let val = f64::from(value);
                running_sum_sq += val * val;

                let count = (i + 1) as f64;
                let mean_val = f64::from(m);
                let variance = (running_sum_sq / count - mean_val * mean_val).max(0.0);
                this.push(V::T::from(variance.sqrt()));
            }

            Ok(())
        })
    }

    /// Compute rolling EMA with variable window starts.
    /// For each index `i`, computes an exponential moving average with
    /// `α = 2/(span+1)` where `span = i - window_starts[i] + 1`.
    pub fn compute_rolling_ema<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<V::T>,
        V::T: From<f64> + Default,
    {
        self.compute_rolling_exponential(max_from, window_starts, values, exit, |span| {
            2.0 / (span + 1.0)
        })
    }

    /// Compute rolling RMA (Wilder's smoothing) with variable window starts.
    /// `α = 1/span` where `span = i - window_starts[i] + 1`.
    pub fn compute_rolling_rma<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<V::T>,
        V::T: From<f64> + Default,
    {
        self.compute_rolling_exponential(max_from, window_starts, values, exit, |span| 1.0 / span)
    }

    fn compute_rolling_exponential<A, F>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        exit: &Exit,
        alpha_fn: F,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<V::T>,
        V::T: From<f64> + Default,
        F: Fn(f64) -> f64,
    {
        self.compute_init(
            Version::new(2) + window_starts.version() + values.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                let mut prev = if skip > 0 {
                    f64::from(this.collect_one_at(skip - 1).unwrap())
                } else {
                    0.0_f64
                };

                let starts_batch = window_starts.collect_range_at(skip, end);
                let values_batch = values.collect_range_at(skip, end);

                for (j, (start, value)) in starts_batch.into_iter().zip(values_batch).enumerate() {
                    let i = skip + j;
                    let span = (i - start.to_usize() + 1) as f64;
                    let alpha = alpha_fn(span);
                    let value = f64::from(value);
                    prev = alpha * value + (1.0 - alpha) * prev;
                    this.push(V::T::from(prev));
                }

                Ok(())
            },
        )
    }

    pub fn compute_sma<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        window: usize,
        exit: &Exit,
        min_i: Option<V::I>,
    ) -> Result<()>
    where
        V::T: Add<V::T, Output = V::T> + From<A> + From<f32>,
        A: VecValue,
        f32: From<V::T> + From<A>,
    {
        self.compute_init(Version::new(2) + source.version(), max_from, exit, |this| {
            let skip = this.len();
            let end = this.batch_end(source.len());
            if skip >= end {
                return Ok(());
            }

            let min_i = min_i.map(|i| i.to_usize());
            let min_prev_i = min_i.unwrap_or_default();

            let mut prev_sma = if skip > 0 && skip > min_prev_i {
                f32::from(this.collect_one_at(skip - 1).unwrap())
            } else {
                0.0
            };

            // Collect only the values that leave the window during this batch.
            // At position i, source[i - window] leaves (when i >= min_prev_i + window).
            // Reads batch_size elements instead of window.
            let pop_start = skip.saturating_sub(window).max(min_prev_i);
            let pop_end = end.saturating_sub(window).max(pop_start);
            let pop_batch: Vec<f32> = if pop_end > pop_start {
                let mut v = Vec::with_capacity(pop_end - pop_start);
                source.for_each_range_dyn_at(pop_start, pop_end, &mut |val: A| {
                    v.push(f32::from(val));
                });
                v
            } else {
                vec![]
            };

            let mut pop_idx = 0;
            let mut i = skip;
            source.fold_range_at(skip, end, (), |(), value: A| {
                if min_i.is_none_or(|m| m <= i) {
                    let value_f32 = f32::from(value);
                    let effective_i = i - min_prev_i;

                    let sma_result = if effective_i >= window {
                        let old = pop_batch[pop_idx];
                        pop_idx += 1;
                        prev_sma + (value_f32 - old) / window as f32
                    } else {
                        (prev_sma * effective_i as f32 + value_f32) / (effective_i + 1) as f32
                    };

                    prev_sma = sma_result;
                    this.push(V::T::from(sma_result));
                } else {
                    this.push(V::T::from(f32::NAN));
                }
                i += 1;
            });
            Ok(())
        })
    }

    fn compute_all_time_extreme<A, F>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        exit: &Exit,
        compare: F,
        exclude_default: bool,
    ) -> Result<()>
    where
        V::T: From<A> + Ord + Default,
        A: VecValue,
        F: Fn(V::T, V::T) -> V::T + Copy,
    {
        let mut prev = None;
        self.compute_transform(
            max_from,
            source,
            |(i, v, this)| {
                let v = V::T::from(v);
                if prev.is_none() {
                    let idx = i.to_usize();
                    prev = Some(if idx > 0 {
                        this.collect_one_at(idx - 1).unwrap()
                    } else {
                        v.clone()
                    });
                }
                let extreme = compare(prev.as_ref().unwrap().clone(), v.clone());

                let next = if !exclude_default || extreme != V::T::default() {
                    extreme
                } else {
                    // Keep the non-default value for future comparisons
                    if v != V::T::default() {
                        v
                    } else {
                        prev.as_ref().unwrap().clone()
                    }
                };
                prev.replace(next.clone());
                (i, next)
            },
            exit,
        )
    }

    /// Computes the all time high of a source.
    pub fn compute_all_time_high<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: From<A> + Ord + Default,
        A: VecValue,
    {
        self.compute_all_time_extreme(max_from, source, exit, |prev, v| prev.max(v), false)
    }

    /// Computes the all time low of a source.
    pub fn compute_all_time_low<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        exit: &Exit,
        exclude_default: bool,
    ) -> Result<()>
    where
        V::T: From<A> + Ord + Default,
        A: VecValue,
    {
        self.compute_all_time_extreme(
            max_from,
            source,
            exit,
            |prev, v| prev.min(v),
            exclude_default,
        )
    }
}
