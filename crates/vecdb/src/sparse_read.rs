use crate::{ReadableVec, VecIndex, VecValue};

/// Batched current/related lookups into one dense source.
///
/// Current indices must be sorted, and every requested source position must
/// exist (sources with holes are not supported). Related indices may be
/// nonmonotonic. Output slots preserve duplicates and missing related indices.
pub struct SparseRead<T: VecValue> {
    slots: Vec<(usize, Option<usize>)>,
    values: Vec<T>,
}

impl<T: VecValue> SparseRead<T> {
    pub fn new<I, R>(
        source: &R,
        indices: &[usize],
        previous: impl FnMut(usize) -> Option<usize>,
    ) -> Self
    where
        I: VecIndex,
        R: ReadableVec<I, T> + ?Sized,
    {
        Self::try_new(source, indices, previous).expect("SparseRead requires a dense source")
    }

    /// Returns `None` if any requested value is absent. Callers accepting
    /// sparse sources can then preserve their scalar/cursor semantics.
    pub fn try_new<I, R>(
        source: &R,
        indices: &[usize],
        mut previous: impl FnMut(usize) -> Option<usize>,
    ) -> Option<Self>
    where
        I: VecIndex,
        R: ReadableVec<I, T> + ?Sized,
    {
        let mut slots: Vec<_> = indices.iter().map(|&i| (i, previous(i))).collect();
        let mut requested = Vec::with_capacity(indices.len() * 2);
        if slots.iter().filter_map(|slot| slot.1).is_sorted() {
            // Merge the sorted current/history streams once and record output
            // slots while merging: no per-output binary searches afterwards.
            let (mut current, mut prior) = (0, 0);
            loop {
                while prior < slots.len() && slots[prior].1.is_none() {
                    prior += 1;
                }
                let a = indices.get(current).copied();
                let b = slots.get(prior).and_then(|slot| slot.1);
                let index = match (a, b) {
                    (Some(a), Some(b)) => a.min(b),
                    (Some(i), None) | (None, Some(i)) => i,
                    (None, None) => break,
                };
                if requested.last() != Some(&index) {
                    requested.push(index);
                }
                let slot = requested.len() - 1;
                if a == Some(index) {
                    slots[current].0 = slot;
                    current += 1;
                }
                if b == Some(index) {
                    slots[prior].1 = Some(slot);
                    prior += 1;
                }
            }
        } else {
            // Custom window metadata need not be monotonic. Preserve the
            // existing general behaviour for those requests.
            requested.extend_from_slice(indices);
            requested.extend(slots.iter().filter_map(|slot| slot.1));
            requested.sort_unstable();
            requested.dedup();
            for slot in &mut slots {
                slot.0 = requested.binary_search(&slot.0).unwrap();
                slot.1 = slot.1.map(|i| requested.binary_search(&i).unwrap());
            }
        }
        let values = source.read_sorted_at(&requested);
        (values.len() == requested.len()).then_some(Self { slots, values })
    }

    #[inline(always)]
    pub fn current(&self, output: usize) -> T {
        self.values[self.slots[output].0].clone()
    }

    #[inline(always)]
    pub fn previous(&self, output: usize) -> Option<T> {
        self.slots[output].1.map(|slot| self.values[slot].clone())
    }
}
