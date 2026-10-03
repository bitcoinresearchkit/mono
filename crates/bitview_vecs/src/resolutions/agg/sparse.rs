use rangeindex::RangeMap;

use vecdb::{ReadableVec, VecIndex, VecValue};

use super::AggFold;

/// Sparse aggregation: emits `Option<T>` per output index.
///
/// `Some(last_value)` when the range contains source elements,
/// `None` when the range is empty.
pub struct Sparse;

impl Sparse {
    fn source_index<SI: VecIndex>(
        mapping: &[SI],
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
        mut f: F,
    ) -> Result<B, E> {
        let source_len = source.visible_len();
        let mapping = mapping.as_slice();

        let mut indices: Vec<usize> = Vec::with_capacity(to - from);
        let mut slot_map: Vec<Option<u32>> = Vec::with_capacity(to - from);

        (from..to).for_each(|idx| {
            if let Some(index) = Self::source_index(mapping, idx, source_len) {
                slot_map.push(Some(indices.len() as u32));
                indices.push(index);
            } else {
                slot_map.push(None);
            }
        });

        let values = source.read_sorted_at(&indices);

        slot_map.iter().try_fold(init, |acc, slot| match slot {
            None => f(acc, None),
            &Some(vi) => f(acc, Some(values[vi as usize].clone())),
        })
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
            Self::source_index(mapping.as_slice(), index, source.visible_len())
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
        let mapping = mapping.as_slice();
        let mut requested = Vec::with_capacity(indices.len());
        let slots: Vec<_> = indices
            .iter()
            .map(|index| {
                Self::source_index(mapping, *index, source_len).map(|index| {
                    if requested.last() != Some(&index) {
                        requested.push(index);
                    }
                    requested.len() - 1
                })
            })
            .collect();
        let values = source.read_sorted_at(&requested);
        out.extend(
            slots
                .into_iter()
                .map(|slot| slot.map(|slot| values[slot].clone())),
        );
    }
}
