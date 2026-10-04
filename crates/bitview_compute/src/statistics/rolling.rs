use std::ops::{AddAssign, SubAssign};

use brk_exit::Exit;
use vecdb::{
    AnyVec, EagerVec, ReadableVec, Result, StoredVec, VecIndex, VecValue, Version, WritableVec,
};

/// Rolling, expanding and all-time statistics computed into an [`EagerVec`].
pub trait ComputeRollingStats {
    type I: VecIndex;
    type T: VecValue;

    /// Compute rolling sum with variable window starts.
    /// For each index i, computes sum of values from `window_starts[i]` to i (inclusive).
    fn compute_rolling_sum<A>(
        &mut self,
        max_from: Self::I,
        window_starts: &impl ReadableVec<Self::I, Self::I>,
        values: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        Self::T: From<A> + Default + AddAssign + SubAssign;

    /// Compute rolling average with variable window starts.
    /// For each index i, computes mean of values from `window_starts[i]` to i (inclusive).
    fn compute_rolling_average<A>(
        &mut self,
        max_from: Self::I,
        window_starts: &impl ReadableVec<Self::I, Self::I>,
        values: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<Self::T>,
        Self::T: From<f64> + Default;

    /// Compute rolling standard deviation with variable window starts.
    /// For each index `i`, computes SD of values from `window_starts[i]` to `i` (inclusive),
    /// using the provided rolling mean.
    /// `SD = sqrt(E[X²] - E[X]²)` where `E[X²]` is the rolling mean of squares
    /// and `E[X]` is the rolling mean from the `mean` parameter.
    fn compute_rolling_sd<A, B>(
        &mut self,
        max_from: Self::I,
        window_starts: &impl ReadableVec<Self::I, Self::I>,
        values: &impl ReadableVec<Self::I, A>,
        mean: &impl ReadableVec<Self::I, B>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        B: VecValue,
        f64: From<A> + From<B> + From<Self::T>,
        Self::T: From<f64>;

    /// Compute rolling EMA with variable window starts.
    /// For each index `i`, computes an exponential moving average with
    /// `α = 2/(span+1)` where `span = i - window_starts[i] + 1`.
    fn compute_rolling_ema<A>(
        &mut self,
        max_from: Self::I,
        window_starts: &impl ReadableVec<Self::I, Self::I>,
        values: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<Self::T>,
        Self::T: From<f64> + Default;

    /// Compute rolling RMA (Wilder's smoothing) with variable window starts.
    /// `α = 1/span` where `span = i - window_starts[i] + 1`.
    fn compute_rolling_rma<A>(
        &mut self,
        max_from: Self::I,
        window_starts: &impl ReadableVec<Self::I, Self::I>,
        values: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<Self::T>,
        Self::T: From<f64> + Default;

    /// Computes the all time high of a source.
    fn compute_all_time_high<A>(
        &mut self,
        max_from: Self::I,
        source: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        Self::T: From<A> + Ord + Default,
        A: VecValue;

    /// Computes the all time low of a source.
    fn compute_all_time_low<A>(
        &mut self,
        max_from: Self::I,
        source: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
        exclude_default: bool,
    ) -> Result<()>
    where
        Self::T: From<A> + Ord + Default,
        A: VecValue;
}

impl<V> ComputeRollingStats for EagerVec<V>
where
    V: StoredVec,
{
    type I = V::I;
    type T = V::T;

    fn compute_rolling_sum<A>(
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

    fn compute_rolling_average<A>(
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
            window_starts.version() + values.version() + Version::new(3),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                // Rebuild the window's sum from its values in f64: recovering it from the stored,
                // rounded average made restarts drift by output-type precision. (A run that never
                // restarts accumulates add/subtract rounding instead; the two differ only at f64 level.)
                let (mut running_sum, mut prev_start) = if skip > 0 {
                    let prev_start = window_starts.collect_one_at(skip - 1).unwrap();
                    if leaving.position() < prev_start.to_usize() {
                        leaving.advance(prev_start.to_usize() - leaving.position());
                    }
                    let sum =
                        values.fold_range_at(prev_start.to_usize(), skip, 0.0, |sum, value| {
                            sum + f64::from(value)
                        });
                    (sum, prev_start)
                } else {
                    (0.0_f64, V::I::from(0))
                };

                let starts_batch = window_starts.collect_range_at(skip, end);
                let values_batch = values.collect_range_at(skip, end);

                for (j, (start, value)) in starts_batch.into_iter().zip(values_batch).enumerate() {
                    let i = skip + j;
                    let value = f64::from(value);
                    // A non-finite value would poison the running sum for good, unlike a resume.
                    debug_assert!(value.is_finite(), "rolling average input must be finite");
                    running_sum += value;

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

    fn compute_rolling_sd<A, B>(
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
            window_starts.version() + values.version() + mean.version() + Version::ONE,
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len()).min(mean.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                // Rebuilt from the window's values in f64, not from the rounded stored outputs.
                let (mut running_sum_sq, mut prev_start) = if skip > 0 {
                    let prev_start = window_starts.collect_one_at(skip - 1).unwrap();
                    if leaving.position() < prev_start.to_usize() {
                        leaving.advance(prev_start.to_usize() - leaving.position());
                    }
                    (
                        values.fold_range_at(prev_start.to_usize(), skip, 0.0, |sum, value| {
                            let value = f64::from(value);
                            sum + value * value
                        }),
                        prev_start,
                    )
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
                    // A non-finite value would poison the running sum for good, unlike a resume.
                    debug_assert!(val.is_finite(), "rolling SD input must be finite");
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

    fn compute_rolling_ema<A>(
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
        compute_rolling_exponential(self, max_from, window_starts, values, exit, |span| {
            2.0 / (span + 1.0)
        })
    }

    fn compute_rolling_rma<A>(
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
        compute_rolling_exponential(self, max_from, window_starts, values, exit, |span| {
            1.0 / span
        })
    }

    fn compute_all_time_high<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: From<A> + Ord + Default,
        A: VecValue,
    {
        compute_all_time_extreme(self, max_from, source, exit, |prev, v| prev.max(v), false)
    }

    fn compute_all_time_low<A>(
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
        compute_all_time_extreme(
            self,
            max_from,
            source,
            exit,
            |prev, v| prev.min(v),
            exclude_default,
        )
    }
}

fn compute_rolling_exponential<V: StoredVec, A, F>(
    vec: &mut EagerVec<V>,
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
    // The stored output is the whole state: it must hold `prev` exactly (an f64-backed type), so a
    // resume continues bit for bit where an uninterrupted run would be.
    vec.compute_init(
        Version::new(3) + window_starts.version() + values.version(),
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
                let out = V::T::from(prev);
                debug_assert!(
                    f64::from(out.clone()) == prev || prev.is_nan(),
                    "exponential average state must be stored exactly"
                );
                this.push(out);
            }

            Ok(())
        },
    )
}

fn compute_all_time_extreme<V: StoredVec, A, F>(
    vec: &mut EagerVec<V>,
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
    vec.compute_transform(
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
