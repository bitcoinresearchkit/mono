use std::{marker::PhantomData, sync::Arc};

mod any_vec;
mod avg;
mod change;
mod clone;
mod op;
mod rate;
mod readable;
mod sub;
mod typed;

pub use avg::DeltaAvg;
pub use change::DeltaChange;
pub use op::DeltaOp;
pub use rate::DeltaRate;
pub use sub::DeltaSub;

use crate::{ReadableBoxedVec, VecIndex, VecValue, Version};

/// Lazily computed vector that combines a source value with a lookback value.
///
/// For each index `h` with `start = window_starts[h]`:
/// - `INCLUSIVE` ops (cumulative source): reads `source[h]` and `source[start - 1]`,
///   count = `h - start + 1`. Used for rolling sums/averages from prefix sums.
/// - Non-inclusive ops (raw source): reads `source[h]` and `source[start]`,
///   count = `h - start`. Used for point-to-point deltas (change, rate).
///
/// Nothing is stored on disk — values are computed on-the-fly during iteration.
pub struct LazyDeltaVec<I: VecIndex, S: VecValue, T, Op> {
    pub(super) name: Arc<str>,
    pub(super) base_version: Version,
    pub(super) source: ReadableBoxedVec<I, S>,
    pub(super) window_starts_version: Version,
    #[allow(clippy::type_complexity)]
    pub(super) window_starts: Arc<dyn Fn() -> Arc<Vec<I>> + Send + Sync>,
    pub(super) _op: PhantomData<(Op, T)>,
}

impl<I, S, T, Op> LazyDeltaVec<I, S, T, Op>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    Op: DeltaOp<S, T>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S>,
        window_starts_version: Version,
        window_starts: impl Fn() -> Arc<Vec<I>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            window_starts_version,
            window_starts: Arc::new(window_starts),
            _op: PhantomData,
        }
    }

    /// Core bulk iteration with fold + early exit support: collect source range
    /// covering both current and ago positions in a single sequential read,
    /// then apply Op per element.
    #[inline]
    pub(super) fn bulk_try_fold<B, E>(
        &self,
        from: usize,
        to: usize,
        starts: &[I],
        init: B,
        mut f: impl FnMut(B, T) -> std::result::Result<B, E>,
    ) -> std::result::Result<B, E> {
        if from >= to {
            return Ok(init);
        }

        // Starts are monotonically non-decreasing, so the earliest ago is from starts[from].
        let read_from = Op::ago_index(starts[from].to_usize())
            .unwrap_or(0)
            .min(from);

        let source_data = self.source.collect_range_dyn(read_from, to);

        let mut acc = init;
        for i in from..to {
            let start = starts[i].to_usize();
            let current = source_data[i - read_from].clone();
            let ago = match Op::ago_index(start) {
                Some(idx) => source_data[idx - read_from].clone(),
                None => Op::ago_default(),
            };
            acc = f(acc, Op::combine(current, ago, Op::count(i, start)))?;
        }
        Ok(acc)
    }

    #[inline]
    pub(super) fn bulk_for_each(
        &self,
        from: usize,
        to: usize,
        starts: &[I],
        mut each: impl FnMut(T),
    ) {
        self.bulk_try_fold(from, to, starts, (), |(), v| {
            each(v);
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap_or_else(|e: std::convert::Infallible| match e {})
    }
}
