use std::{marker::PhantomData, sync::Arc};

pub mod any_vec;
pub mod avg;
pub mod change;
pub mod clone;
pub mod op;
pub mod rate;
pub mod readable;
pub mod sub;
pub mod typed;

pub use avg::DeltaAvg;
pub use change::DeltaChange;
pub use op::DeltaOp;
pub use rate::DeltaRate;
pub use sub::DeltaSub;

use crate::{
    AnyVec, CheckedSub, ReadableBoxedVec, VecIndex, VecValue, Version, WindowBatch, WindowBuffers,
    before_window_bounds, read_batches,
};

/// `current - before` as `f64`, subtracted exactly first: cumulative integers can exceed `f64`'s
/// 53 bits, where converting both operands first loses the difference. A difference past `S`'s
/// range (signed extremes) is subtracted in `f64` instead.
#[inline(always)]
fn exact_diff<S: CheckedSub + PartialOrd + Clone + Into<f64>>(current: S, before: S) -> f64 {
    let (high, low, sign) = if current >= before {
        (current, before, 1.0)
    } else {
        (before, current, -1.0)
    };
    let diff = high
        .clone()
        .checked_sub(low.clone())
        .map_or_else(|| high.into() - low.into(), Into::into);
    sign * diff
}

/// Lazily computed vector that combines a source value with the value before its window.
///
/// For each index `h` with `start = window_starts[h]` (the window's first index), `Op` combines
/// `source[h]` with the value before the window, `source[start - 1]` (the default for a window
/// from 0), and the count [`DeltaOp::count`] gives (by default the window length
/// `h - start + 1`): rolling sums/averages from prefix sums, and changes that equal the sum of
/// the flows in the window.
///
/// Sources are dense (every index below their length holds a value) and never shrink while read:
/// they may grow, but a truncation waits for readers (checked in dev builds).
pub struct LazyDeltaVec<I: VecIndex, S: VecValue, T, Op> {
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    window_starts: ReadableBoxedVec<I, I>,
    _op: PhantomData<(Op, T)>,
}

impl<I, S, T, Op> LazyDeltaVec<I, S, T, Op>
where
    I: VecIndex,
    S: VecValue + Default,
    T: VecValue,
    Op: DeltaOp<S, T>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S>,
        window_starts: ReadableBoxedVec<I, I>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            window_starts,
            _op: PhantomData,
        }
    }

    /// Visits `[from, to)` in [`read_batches`], each with its window starts and its source batch
    /// ([`WindowBuffers`]). Scratch covers the batch and the span of its before-window values.
    fn try_for_each_batch<E>(
        &self,
        from: usize,
        to: usize,
        mut visit: impl FnMut(usize, &[I], WindowBatch<'_, S>) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        let (mut starts, mut buffers) = (Vec::new(), WindowBuffers::default());
        // Batches large enough that historical pages shared by adjacent ones are rarely read twice.
        for (at, end) in read_batches(from, to, self.source.cursor_chunk_size()) {
            starts.clear();
            self.window_starts.read_into_at(at, end, &mut starts);
            debug_assert_eq!(starts.len(), end - at, "LazyDeltaVec sources are dense");
            let bounds = before_window_bounds(&starts);
            visit(at, &starts, buffers.read(&*self.source, at, end, bounds))?;
        }
        Ok(())
    }

    fn transformed_values<'a>(
        at: usize,
        starts: &'a [I],
        batch: WindowBatch<'a, S>,
    ) -> impl Iterator<Item = T> + 'a {
        (at..)
            .zip(batch.current)
            .zip(starts)
            .map(move |((index, current), start)| {
                let start = start.to_usize();
                Op::combine(
                    current.clone(),
                    batch.before(start),
                    Op::count(index, start),
                )
            })
    }
}
