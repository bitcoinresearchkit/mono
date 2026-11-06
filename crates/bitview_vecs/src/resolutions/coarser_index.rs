use std::marker::PhantomData;

use bitview_primitives::FromCoarserIndex;
use rangeindex::RangeMap;
use vecdb::{ReadableVec, VecIndex, VecValue};

use crate::AggFold;

/// Aggregation strategy for epoch-based indices: each epoch's value is the source's at the
/// epoch's last height, or at the source's last row for the epoch it is in. The output stops at
/// the epochs the source reaches, so a source behind the mapping never answers a later epoch
/// with an earlier one's value.
pub struct CoarserIndex<I>(PhantomData<I>);

impl<I, O, S1I> AggFold<O, S1I, O> for CoarserIndex<I>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex + FromCoarserIndex<I>,
{
    #[inline]
    fn len<MI: VecIndex, S: ReadableVec<S1I, O> + ?Sized>(
        source: &S,
        mapping: &RangeMap<S1I, MI>,
    ) -> usize {
        mapping.reached(source.visible_len())
    }

    #[inline]
    fn try_fold<
        MI: VecIndex,
        S: ReadableVec<S1I, O> + ?Sized,
        B,
        E,
        F: FnMut(B, O) -> Result<B, E>,
    >(
        source: &S,
        _mapping: &RangeMap<S1I, MI>,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        let source_len = source.visible_len();

        let indices: Vec<usize> = (from..to)
            .map(|i| S1I::max_from(I::from(i), source_len))
            .collect();

        source
            .read_sorted_at(&indices)
            .into_iter()
            .try_fold(init, f)
    }

    #[inline]
    fn collect_one<MI: VecIndex, S: ReadableVec<S1I, O> + ?Sized>(
        source: &S,
        _mapping: &RangeMap<S1I, MI>,
        index: usize,
    ) -> Option<O> {
        let target = S1I::max_from(I::from(index), source.visible_len());
        source.collect_one_at(target)
    }

    /// One sorted source read for all requested epochs instead of a point read each.
    fn read_sorted_into<MI: VecIndex, S: ReadableVec<S1I, O> + ?Sized>(
        source: &S,
        _mapping: &RangeMap<S1I, MI>,
        indices: &[usize],
        out: &mut Vec<O>,
    ) {
        let source_len = source.visible_len();
        let targets: Vec<usize> = indices
            .iter()
            .map(|&index| S1I::max_from(I::from(index), source_len))
            .collect();
        source.read_sorted_into_at(&targets, out);
    }
}
