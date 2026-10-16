use std::{
    collections::VecDeque,
    ops::{AddAssign, SubAssign},
};

use brk_exit::Exit;
use vecdb::{ComputeVec, ReadableVec, Result, StoredVec, VecIndex, VecValue, Version};

use super::Retained;

/// Rolling, expanding and all-time statistics computed into a stored vec.
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
    /// up past one stride. The `retained` window continues the previous update when current.
    fn compute_rolling_average<A>(
        &mut self,
        max_from: Self::I,
        window_starts: &impl ReadableVec<Self::I, Self::I>,
        values: &impl ReadableVec<Self::I, A>,
        retained: &mut Retained<AverageWindow>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<Self::T>,
        Self::T: From<f64> + Default;

    /// Compute rolling EMA with variable window starts.
    /// For each index `i`, computes an exponential moving average with
    /// `α = 2/(span+1)` where `span = i - window_starts[i] + 1`. A one-row window seeds the
    /// average with its value; a window starting after its row is empty: NaN.
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

    /// Computes the all time low of a source's values above the default: zero and undefined
    /// values (which sort below it) are skipped, and the low stays at the first value until one
    /// above the default arrives.
    fn compute_all_time_low<A>(
        &mut self,
        max_from: Self::I,
        source: &impl ReadableVec<Self::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        Self::T: From<A> + Ord + Default,
        A: VecValue;
}

/// Rows at multiples of this rebuild their window's sum from its values.
const ANCHOR_STRIDE: usize = 1024;

/// A rolling average's window values in row order and their running f64 sum: the state of
/// [`ComputeRollingStats::compute_rolling_average`].
pub struct AverageWindow {
    /// The window's values, the first at row `start`.
    values: VecDeque<f64>,
    start: usize,
    sum: f64,
}

impl AverageWindow {
    /// The values of the window of row `anchor` before `anchor` itself, from history. The
    /// sum is left to the anchor row, which rebuilds it.
    fn read<I, A>(
        anchor: usize,
        window_starts: &impl ReadableVec<I, I>,
        source: &impl ReadableVec<I, A>,
    ) -> Self
    where
        I: VecIndex,
        A: VecValue,
        f64: From<A>,
    {
        debug_assert!(anchor.is_multiple_of(ANCHOR_STRIDE));
        let start = window_starts.collect_one_at(anchor).unwrap().to_usize();
        let mut values = VecDeque::with_capacity(anchor - start);
        source.for_each_range_at(start, anchor, |value| values.push_back(f64::from(value)));
        Self {
            values,
            start,
            sum: 0.0,
        }
    }

    /// Enters row `index`'s value, drops the rows before its window `start`, and returns the
    /// window's mean. A row at a multiple of [`ANCHOR_STRIDE`] rebuilds the sum from its
    /// window's values; the rows after add the entering value and remove the leaving ones.
    #[inline]
    fn push(&mut self, index: usize, value: f64, start: usize) -> f64 {
        debug_assert_eq!(index, self.start + self.values.len(), "rows enter in order");
        debug_assert!(self.start <= start, "window starts never move back");
        self.values.push_back(value);
        if index.is_multiple_of(ANCHOR_STRIDE) {
            self.values.drain(..start - self.start);
            self.start = start;
            self.sum = self
                .values
                .iter()
                .fold(0.0, |sum, &value| sum + defined(value));
        } else {
            self.sum += defined(value);
            while self.start < start {
                self.sum -= defined(self.values.pop_front().unwrap());
                self.start += 1;
            }
        }
        self.sum / (index - start + 1) as f64
    }
}

impl<V> ComputeRollingStats for V
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

        self.compute_to(
            max_from,
            window_starts.len().min(values.len()),
            window_starts.version() + values.version(),
            |this, range| {
                let (skip, end) = (range.start, range.end);

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
            exit,
        )
    }

    fn compute_rolling_average<A>(
        &mut self,
        max_from: V::I,
        window_starts: &impl ReadableVec<V::I, V::I>,
        values: &impl ReadableVec<V::I, A>,
        retained: &mut Retained<AverageWindow>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecValue,
        f64: From<A> + From<V::T>,
        V::T: From<f64> + Default,
    {
        let version = Version::new(4) + window_starts.version() + values.version();
        self.compute_to(
            max_from,
            window_starts.len().min(values.len()),
            version,
            |this, range| {
                let (skip, end) = (range.start, range.end);
                // Without a current window, replay from the anchor at or before `skip`.
                let (mut window, from) = match retained.take(skip, version) {
                    Some(window) => (window, skip),
                    None => {
                        let anchor = skip - skip % ANCHOR_STRIDE;
                        (AverageWindow::read(anchor, window_starts, values), anchor)
                    }
                };
                let starts = window_starts.collect_range_at(from, end);
                let entering = values.collect_range_at(from, end);
                for (index, (start, value)) in (from..end).zip(starts.into_iter().zip(entering)) {
                    let average = window.push(index, f64::from(value), start.to_usize());
                    if index >= skip {
                        this.push(V::T::from(average));
                    }
                }
                retained.keep(end, version, window);
                Ok(())
            },
            exit,
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
        self.compute_to(
            max_from,
            window_starts.len().min(values.len()),
            Version::new(4) + window_starts.version() + values.version(),
            |this, range| {
                let (skip, end) = (range.start, range.end);

                let mut prev = if skip > 0 {
                    f64::from(this.collect_one_at(skip - 1).unwrap())
                } else {
                    0.0_f64
                };

                let starts_batch = window_starts.collect_range_at(skip, end);
                let values_batch = values.collect_range_at(skip, end);

                for (i, (start, value)) in (skip..).zip(starts_batch.into_iter().zip(values_batch))
                {
                    let value = f64::from(value);
                    prev = match (i + 1).saturating_sub(start.to_usize()) {
                        0 => f64::NAN,
                        1 => value,
                        span => {
                            let alpha = 2.0 / (span as f64 + 1.0);
                            alpha * value + (1.0 - alpha) * prev
                        }
                    };
                    let out = V::T::from(prev);
                    debug_assert!(
                        f64::from(out.clone()) == prev || prev.is_nan(),
                        "exponential average state must be stored exactly"
                    );
                    this.push(out);
                }

                Ok(())
            },
            exit,
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
        compute_all_time_extreme(self, max_from, source, exit, |prev, v| prev.max(v))
    }

    fn compute_all_time_low<A>(
        &mut self,
        max_from: V::I,
        source: &impl ReadableVec<V::I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        V::T: From<A> + Ord + Default,
        A: VecValue,
    {
        compute_all_time_extreme(self, max_from, source, exit, |prev, v| {
            let undefined = V::T::default();
            if v <= undefined {
                prev
            } else if prev <= undefined {
                v
            } else {
                prev.min(v)
            }
        })
    }
}

fn compute_all_time_extreme<V: StoredVec, A, F>(
    vec: &mut V,
    max_from: V::I,
    source: &impl ReadableVec<V::I, A>,
    exit: &Exit,
    compare: F,
) -> Result<()>
where
    V::T: From<A> + Ord + Default,
    A: VecValue,
    F: Fn(V::T, V::T) -> V::T,
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
            let next = compare(prev.take().unwrap(), v);
            prev = Some(next.clone());
            next
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
