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

#[cfg(test)]
mod tests {
    use rangeindex::RangeMap;
    use tempfile::tempdir;

    use super::{AggFold, Sparse};
    use vecdb::{BytesVec, Database, ImportableVec, ReadBounds, Version, WritableVec};

    fn mapping_source(values: &[usize]) -> RangeMap<usize, usize> {
        RangeMap::from(values.to_vec())
    }

    #[test]
    fn sparse_ranges_and_early_errors_match_reference_buckets() {
        let temp = tempdir().unwrap();
        let db = Database::open(temp.path()).unwrap();
        let mut source: BytesVec<usize, u64> =
            BytesVec::forced_import(&db, "buckets", Version::ONE).unwrap();
        let values = [10, 20, 30, 40];
        for value in values {
            source.push(value);
        }

        for mapping in [
            vec![],
            vec![0],
            vec![0, 0, 0, 0],
            vec![0, 1, 2, 3],
            vec![0, 2, 2, 3, 100],
            vec![0, 0, 4, 4, 4],
            vec![2, 2, 3],
        ] {
            for source_len in [0, 2, values.len()] {
                let expected: Vec<_> = mapping
                    .iter()
                    .enumerate()
                    .map(|(i, &first)| {
                        let end = mapping
                            .get(i + 1)
                            .copied()
                            .unwrap_or(source_len)
                            .min(source_len);
                        (first..end).last().map(|index| values[index])
                    })
                    .collect();
                let mut bounds = ReadBounds::new();
                let mapping = mapping_source(&mapping);
                bounds.set("usize", source_len);
                bounds.scope(|| {
                    for (index, &expected) in expected.iter().enumerate() {
                        assert_eq!(
                            Sparse::collect_one(&source, &mapping, index),
                            Some(expected)
                        );
                    }
                    for index in [mapping.len(), usize::MAX] {
                        assert_eq!(Sparse::collect_one(&source, &mapping, index), None);
                    }
                    for from in 0..=mapping.len() {
                        for to in from..=mapping.len() {
                            let actual = Sparse::fold(
                                &source,
                                &mapping,
                                from,
                                to,
                                Vec::new(),
                                |mut acc, value| {
                                    acc.push(value);
                                    acc
                                },
                            );
                            assert_eq!(actual, expected[from..to]);
                            for stop in 0..=to - from {
                                let mut visited = Vec::new();
                                let result = Sparse::try_fold(
                                    &source,
                                    &mapping,
                                    from,
                                    to,
                                    (),
                                    |(), value| {
                                        visited.push(value);
                                        if visited.len() > stop {
                                            Err(stop)
                                        } else {
                                            Ok(())
                                        }
                                    },
                                );
                                assert_eq!(
                                    result,
                                    if stop < to - from { Err(stop) } else { Ok(()) }
                                );
                                assert_eq!(visited, expected[from..to.min(from + stop + 1)]);
                            }
                        }
                    }
                });
            }
        }
    }
}
