use rangeindex::RangeMap;

use vecdb::{ReadableVec, VecIndex, VecValue};

use super::{AggFold, read_sorted_optional};

/// Sparse aggregation: emits `Option<T>` per output index.
///
/// `Some(last_value)` when the range contains source elements,
/// `None` when the range is empty.
pub struct Sparse;

impl<T: VecValue, SI: VecIndex> AggFold<Option<T>, SI, T> for Sparse {
    #[inline]
    fn try_fold<
        MI: VecIndex,
        S: ReadableVec<SI, T> + ?Sized,
        B,
        E,
        F: FnMut(B, Option<T>) -> Result<B, E>,
    >(
        source: &S,
        mapping: &RangeMap<SI, MI>,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        let source_len = source.visible_len();
        read_sorted_optional(
            source,
            (from..to).map(|index| mapping.range(index, source_len).next_back()),
            false,
        )
        .try_fold(init, f)
    }

    #[inline]
    fn collect_one<MI: VecIndex, S: ReadableVec<SI, T> + ?Sized>(
        source: &S,
        mapping: &RangeMap<SI, MI>,
        index: usize,
    ) -> Option<Option<T>> {
        if index >= mapping.len() {
            return None;
        }
        Some(
            mapping
                .range(index, source.visible_len())
                .next_back()
                .and_then(|i| source.collect_one_at(i)),
        )
    }

    fn read_sorted_into<MI: VecIndex, S: ReadableVec<SI, T> + ?Sized>(
        source: &S,
        mapping: &RangeMap<SI, MI>,
        indices: &[usize],
        out: &mut Vec<Option<T>>,
    ) {
        if let &[index] = indices {
            out.push(Self::collect_one(source, mapping, index).unwrap());
            return;
        }
        let source_len = source.visible_len();
        out.extend(read_sorted_optional(
            source,
            indices
                .iter()
                .map(|&index| mapping.range(index, source_len).next_back()),
            true,
        ));
    }
}
