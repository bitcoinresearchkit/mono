use std::sync::Arc;

use crate::{AnyVec, ReadOnlyClone, ReadableVec, TypedVec, VecIndex, VecValue, Version};

/// Values generated from their index, including constants that ignore it.
/// The source supplies live length/version metadata; its values are never read.
#[derive(Clone)]
pub struct IndexVec<I, T, S> {
    name: Arc<str>,
    base_version: Version,
    source: S,
    compute: fn(I) -> T,
}

impl<I: VecIndex, T: VecValue, S: TypedVec<I = I>> IndexVec<I, T, S> {
    pub fn new(name: &str, version: Version, source: S, compute: fn(I) -> T) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            compute,
        }
    }

    fn values(&self, from: usize, to: usize) -> impl Iterator<Item = T> + '_ {
        (from..to.min(self.visible_len())).map(|i| (self.compute)(I::from(i)))
    }
}

impl<I: VecIndex, T: VecValue, S: TypedVec<I = I>> AnyVec for IndexVec<I, T, S> {
    fn name(&self) -> &str {
        &self.name
    }
    fn version(&self) -> Version {
        self.base_version + self.source.version()
    }
    fn len(&self) -> usize {
        self.source.len()
    }
    fn visible_len(&self) -> usize {
        self.source.visible_len()
    }

    crate::any_vec_type_methods!(I, T);
}

impl<I: VecIndex, T: VecValue, S: TypedVec<I = I>> TypedVec for IndexVec<I, T, S> {
    type I = I;
    type T = T;
}

impl<I: VecIndex, T: VecValue, S: TypedVec<I = I> + Clone> ReadOnlyClone for IndexVec<I, T, S> {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl<I: VecIndex, T: VecValue, S: TypedVec<I = I>> ReadableVec<I, T> for IndexVec<I, T, S> {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        out.extend(self.values(from, to));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.values(from, to).for_each(f);
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B {
        self.values(from, to).fold(init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        self.values(from, to).try_fold(init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        (index < self.visible_len()).then(|| (self.compute)(I::from(index)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let len = self.visible_len();
        out.extend(
            indices
                .iter()
                .copied()
                .take_while(|&i| i < len)
                .map(|i| (self.compute)(I::from(i))),
        );
    }
}
