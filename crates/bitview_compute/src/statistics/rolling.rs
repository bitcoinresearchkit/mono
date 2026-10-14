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
    /// For each index i, computes mean of values from `window_starts[i]` to i (inclusive); an
    /// undefined value counts as zero. Every 1,024th row rebuilds its window's f64 sum from the
    /// values and the rows after continue from it, so a row's value depends on its position
    /// alone: a resumed run matches an uninterrupted one bit for bit, and rounding never builds
    /// up past one stride.
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

/// Rows at multiples of this rebuild their window's sum from its values.
const ANCHOR_STRIDE: usize = 1024;

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
        // Leaving values are read through one cursor across batches, so each compressed page is
        // decompressed once; only a replay from an anchor behind it starts a new one.
        let mut leaving = values.cursor();

        self.compute_init(
            Version::new(4) + window_starts.version() + values.version(),
            max_from,
            exit,
            |this| {
                let skip = this.len();
                let source_len = window_starts.len().min(values.len());
                let end = this.batch_end(source_len);
                if skip >= end {
                    return Ok(());
                }

                let anchor = skip - skip % ANCHOR_STRIDE;
                let starts_batch = window_starts.collect_range_at(anchor, end);
                let values_batch = values.collect_range_at(anchor, end);
                let mut sum = 0.0;
                let mut prev_start = 0;

                for (j, (start, value)) in starts_batch.into_iter().zip(values_batch).enumerate() {
                    let i = anchor + j;
                    let start = start.to_usize();
                    debug_assert!(prev_start <= start, "window starts never move back");
                    if i.is_multiple_of(ANCHOR_STRIDE) {
                        sum = values.fold_range_at(start, i + 1, 0.0, |sum, value| {
                            sum + defined(f64::from(value))
                        });
                        if leaving.position() > start {
                            leaving = values.cursor();
                        }
                        leaving.advance(start - leaving.position());
                    } else {
                        sum += defined(f64::from(value));
                        if prev_start < start {
                            leaving.for_each(start - prev_start, |value: A| {
                                sum -= defined(f64::from(value));
                            });
                        }
                    }
                    prev_start = start;
                    if i >= skip {
                        this.push(V::T::from(sum / (i - start + 1) as f64));
                    }
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
        // The stored output is the whole state: it must hold `prev` exactly (an f64-backed type), so a
        // resume continues bit for bit where an uninterrupted run would be.
        self.compute_init(
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
                    let alpha = 2.0 / (span + 1.0);
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

/// Rolling window statistics kept as f64 sums of the window's values (`add`, `remove`), written
/// by `output` from the sums and the window's count. A row at a multiple of [`ANCHOR_STRIDE`]
/// rebuilds the sums from its window; the rows after continue from it, adding the entering value
/// and removing the leaving ones. A row's value thus depends on its position alone: a run
/// resuming anywhere replays from the anchor at or before it, matching an uninterrupted run bit
/// for bit, and rounding never builds up past one stride.
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

/// An undefined (non-finite) value counts as zero in a running sum, which it would otherwise
/// poison for good.
#[inline(always)]
fn defined(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}
