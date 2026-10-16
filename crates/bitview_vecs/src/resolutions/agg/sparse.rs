use rangeindex::{Pages, RangeMap};

use vecdb::{ReadableVec, VecIndex, VecValue};

use super::{AggFold, read_sorted_optional};

/// Sparse aggregation: emits `Option<T>` per output index.
///
/// `Some(last_value)` when the range contains source elements,
/// `None` when the range is empty.
pub struct Sparse;

impl Sparse {
    fn source_index<SI: VecIndex>(
        mapping: &Pages<SI>,
        index: usize,
        source_len: usize,
    ) -> Option<usize> {
        let first = mapping[index].to_usize();
        let end = mapping
            .get(index + 1)
            .map(|i| i.to_usize())
            .unwrap_or(source_len)
            .min(source_len);
        (first < end).then(|| end - 1)
    }
}

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
        let mapping = mapping.starts();
        read_sorted_optional(
            source,
            (from..to).map(|index| Self::source_index(mapping, index, source_len)),
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
            Self::source_index(mapping.starts(), index, source.visible_len())
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
        let mapping = mapping.starts();
        out.extend(read_sorted_optional(
            source,
            indices
                .iter()
                .map(|&index| Self::source_index(mapping, index, source_len)),
            true,
        ));
    }
}
