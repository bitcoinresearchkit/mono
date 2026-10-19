use std::convert::Infallible;

use rangeindex::RangeMap;

use vecdb::{ReadableVec, VecIndex, VecValue};

/// Aggregation strategy for [`super::LazyAggVec`].
///
/// Determines how values are produced from a source vec and a range map.
/// Implement this on a zero-sized marker type to define a custom strategy.
///
/// `Sparse` (the last value, or none) is the built-in one.
pub trait AggFold<O: VecValue, S1I: VecIndex, S1T: VecValue>: 'static {
    fn try_fold<
        MI: VecIndex,
        S: ReadableVec<S1I, S1T> + ?Sized,
        B,
        E,
        F: FnMut(B, O) -> Result<B, E>,
    >(
        source: &S,
        mapping: &RangeMap<S1I, MI>,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E>;

    fn fold<MI: VecIndex, S: ReadableVec<S1I, S1T> + ?Sized, B, F: FnMut(B, O) -> B>(
        source: &S,
        mapping: &RangeMap<S1I, MI>,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let Ok(b) = Self::try_fold(source, mapping, from, to, init, |b, o| {
            Ok::<_, Infallible>(f(b, o))
        });
        b
    }

    fn collect_one<MI: VecIndex, S: ReadableVec<S1I, S1T> + ?Sized>(
        source: &S,
        mapping: &RangeMap<S1I, MI>,
        index: usize,
    ) -> Option<O> {
        let mut result = None;
        Self::fold(source, mapping, index, index + 1, (), |(), v| {
            result = Some(v)
        });
        result
    }

    /// Evaluate only the requested output buckets. Strategies can batch their
    /// source lookups without computing adjacent, unrequested buckets.
    fn read_sorted_into<MI: VecIndex, S: ReadableVec<S1I, S1T> + ?Sized>(
        source: &S,
        mapping: &RangeMap<S1I, MI>,
        indices: &[usize],
        out: &mut Vec<O>,
    ) {
        out.reserve(indices.len());
        for &index in indices {
            if let Some(value) = Self::collect_one(source, mapping, index) {
                out.push(value);
            }
        }
    }
}

/// One sorted read of `source` at ascending `positions` (`None` reads nothing), yielding each
/// position's value in order. With `repeats`, equal adjacent positions may occur and share one
/// read; strictly increasing positions pass `false` and skip that check.
#[inline]
pub fn read_sorted_optional<I, T, S>(
    source: &S,
    positions: impl ExactSizeIterator<Item = Option<usize>>,
    repeats: bool,
) -> impl Iterator<Item = Option<T>>
where
    I: VecIndex,
    T: VecValue,
    S: ReadableVec<I, T> + ?Sized,
{
    let mut requested = Vec::with_capacity(positions.len());
    let slots: Vec<Option<u32>> = positions
        .map(|position| {
            position.map(|position| {
                if !repeats || requested.last() != Some(&position) {
                    requested.push(position);
                }
                (requested.len() - 1) as u32
            })
        })
        .collect();
    let values = source.read_sorted_at(&requested);
    slots
        .into_iter()
        .map(move |slot| slot.map(|slot| values[slot as usize].clone()))
}
