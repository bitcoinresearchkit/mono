use vecdb::{ReadableVec, VecIndex, VecValue};

pub struct SparseRead<T: VecValue> {
    indices: Vec<usize>,
    values: Vec<T>,
}

impl<T: VecValue> SparseRead<T> {
    pub fn new<I, R>(source: &R, indices: impl IntoIterator<Item = usize>) -> Self
    where
        I: VecIndex,
        R: ReadableVec<I, T> + ?Sized,
    {
        let mut indices: Vec<_> = indices.into_iter().collect();
        indices.sort_unstable();
        indices.dedup();
        let values = source.read_sorted_at(&indices);
        debug_assert_eq!(values.len(), indices.len());

        Self { indices, values }
    }

    #[inline(always)]
    pub fn at(&self, index: usize) -> T {
        self.values[self.indices.binary_search(&index).unwrap()].clone()
    }
}
