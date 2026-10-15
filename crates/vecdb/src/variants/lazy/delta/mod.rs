use std::{convert::Infallible, marker::PhantomData, sync::Arc};

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

use crate::{AnyVec, ReadableBoxedVec, VecIndex, VecValue, Version, read_batches};

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
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    #[allow(clippy::type_complexity)]
    window_starts: ReadableBoxedVec<I, I>,
    _op: PhantomData<(Op, T)>,
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

    /// Visits `[from, to)` in [`read_batches`], each with its current
    /// values, window starts and the historical values they reach (read from `previous_from`).
    /// Overlapping current and historical ranges are read once; the gap between disjoint ones
    /// never is. Each batch's reads complete before `visit` runs: sources may hold
    /// non-reentrant publication locks while lending chunks. Scratch is bounded by the batch.
    fn try_for_each_batch<E>(
        &self,
        from: usize,
        to: usize,
        mut visit: impl FnMut(usize, &[S], &[I], usize, &[S]) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        let (mut starts, mut values, mut previous) = (Vec::new(), Vec::new(), Vec::new());
        // Batches large enough that historical pages shared by adjacent ones are rarely read twice.
        for (at, end) in read_batches(from, to, self.source.cursor_chunk_size()) {
            starts.clear();
            self.window_starts.read_into_at(at, end, &mut starts);
            // Window starts need not be monotonic: bound every historical index.
            let bounds = starts
                .iter()
                .filter_map(|start| Op::ago_index(start.to_usize()))
                .fold(None, |bounds: Option<(usize, usize)>, index| {
                    Some(bounds.map_or((index, index), |(low, high)| {
                        (low.min(index), high.max(index))
                    }))
                });
            values.clear();
            match bounds {
                Some((low, high)) if high.saturating_add(1) >= at => {
                    let read_from = low.min(at);
                    self.source.read_into_at(read_from, end, &mut values);
                    visit(at, &values[at - read_from..], &starts, read_from, &values)?;
                }
                Some((low, high)) => {
                    previous.clear();
                    self.source.read_into_at(low, high + 1, &mut previous);
                    self.source.read_into_at(at, end, &mut values);
                    visit(at, &values, &starts, low, &previous)?;
                }
                None => {
                    self.source.read_into_at(at, end, &mut values);
                    visit(at, &values, &starts, 0, &[])?;
                }
            }
        }
        Ok(())
    }

    fn transformed_values<'a>(
        at: usize,
        current: &'a [S],
        starts: &'a [I],
        previous_from: usize,
        previous: &'a [S],
    ) -> impl Iterator<Item = T> + 'a {
        (at..at + current.len())
            .zip(current)
            .zip(starts)
            .map(move |((index, current), start)| {
                let start = start.to_usize();
                let ago = Op::ago_index(start)
                    .map(|index| previous[index - previous_from].clone())
                    .unwrap_or_else(Op::ago_default);
                Op::combine(current.clone(), ago, Op::count(index, start))
            })
    }

    /// Scalar transform/consumer ordering is retained for fallible folds.
    #[inline]
    fn bulk_try_fold<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_batch(from, to, |at, current, starts, previous_from, previous| {
            acc = Some(
                Self::transformed_values(at, current, starts, previous_from, previous)
                    .try_fold(acc.take().unwrap(), &mut f)?,
            );
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    #[inline]
    fn bulk_fold<B>(&self, from: usize, to: usize, init: B, mut f: impl FnMut(B, T) -> B) -> B {
        let Ok(acc) = self.bulk_try_fold(from, to, init, |acc, value| {
            Ok::<_, Infallible>(f(acc, value))
        });
        acc
    }

    /// Visits each batch's transformed values through one reused buffer.
    fn for_each_transformed_chunk(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let mut output = Vec::new();
        let Ok(()) =
            self.try_for_each_batch(from, to, |at, current, starts, previous_from, previous| {
                output.clear();
                output.extend(Self::transformed_values(
                    at,
                    current,
                    starts,
                    previous_from,
                    previous,
                ));
                f(at, &output);
                Ok::<_, Infallible>(())
            });
    }
}
