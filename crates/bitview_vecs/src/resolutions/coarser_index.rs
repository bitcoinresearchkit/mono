use std::marker::PhantomData;

use bitview_primitives::FromCoarserIndex;
use rangeindex::RangeMap;
use vecdb::{ReadableVec, VecIndex, VecValue};

use crate::AggFold;

/// Aggregation strategy for epoch-based indices.
///
/// The mapping supplies the output length while the index determines the
/// corresponding source height.
pub struct CoarserIndex<I>(PhantomData<I>);

impl<I, O, S1I> AggFold<O, S1I, O> for CoarserIndex<I>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex + FromCoarserIndex<I>,
{
    #[inline]
    fn try_fold<
        MI: VecIndex,
        S: ReadableVec<S1I, O> + ?Sized,
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
    ) -> Result<B, E> {
        let mapping_len = mapping.len();
        let source_len = source.visible_len();

        let indices: Vec<usize> = (from..to.min(mapping_len))
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
}
