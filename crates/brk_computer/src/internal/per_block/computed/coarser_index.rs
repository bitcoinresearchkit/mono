use std::marker::PhantomData;

use brk_types::FromCoarserIndex;
use vecdb::{AggFold, ReadableVec, VecIndex, VecValue};

/// Aggregation strategy for epoch-based indices.
///
/// The mapping supplies the output length while the index determines the
/// corresponding source height.
pub struct CoarserIndex<I>(PhantomData<I>);

impl<I, O, S1I, S2T> AggFold<O, S1I, S2T, O> for CoarserIndex<I>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex + FromCoarserIndex<I>,
    S2T: VecValue,
{
    #[inline]
    fn try_fold<S: ReadableVec<S1I, O> + ?Sized, B, E, F: FnMut(B, O) -> Result<B, E>>(
        source: &S,
        mapping: &[S2T],
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
    fn collect_one<S: ReadableVec<S1I, O> + ?Sized>(
        source: &S,
        _mapping: &[S2T],
        index: usize,
    ) -> Option<O> {
        let target = S1I::max_from(I::from(index), source.visible_len());
        source.collect_one_at(target)
    }
}
