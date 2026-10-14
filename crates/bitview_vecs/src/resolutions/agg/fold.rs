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
        match Self::try_fold(source, mapping, from, to, init, |b, o| {
            Ok::<_, Infallible>(f(b, o))
        }) {
            Ok(b) => b,
            Err(e) => match e {},
        }
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
