use std::sync::Arc;

use arc_swap::Guard;
use brk_types::{Height, Version};
use rangeindex::{RangeMap, SharedRangeMap};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{AnyVec, Formattable, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex, VecValue};

/// Read-only vector view of a shared range map derived from the height mapping `source`, whose
/// live version it carries (like a lazy view). Updates belong to the map owner; this view adds
/// series metadata and reads.
#[derive(Clone)]
pub struct RangeMapVec<I: VecIndex, T> {
    name: Arc<str>,
    version: Version,
    source: ReadableBoxedVec<Height, I>,
    mapping: SharedRangeMap<T, I>,
}

impl<I: VecIndex, T: VecValue> RangeMapVec<I, T> {
    /// The view's version is `version` plus `source`'s.
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, I>,
        mapping: SharedRangeMap<T, I>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            version,
            source,
            mapping,
        }
    }

    /// The latest published map; load it once per read.
    #[inline]
    pub(crate) fn load(&self) -> Guard<Arc<RangeMap<T, I>>> {
        self.mapping.load()
    }
}

impl<I: VecIndex, T: VecValue> AnyVec for RangeMapVec<I, T> {
    fn version(&self) -> Version {
        self.version + self.source.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.load().len()
    }

    /// The map only grows, but for a rollback rewriting it under the publication.
    fn is_mutable(&self) -> bool {
        false
    }

    vecdb::any_vec_type_methods!(I, T);
}

impl<I: VecIndex, T: VecValue> TypedVec for RangeMapVec<I, T> {
    type I = I;
    type T = T;
}

impl<I: VecIndex, T: VecValue + Copy> ReadableVec<I, T> for RangeMapVec<I, T> {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        let index = self.load();
        for (_, values) in index.starts().chunks(from..to) {
            out.extend_from_slice(values);
        }
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, each: &mut dyn FnMut(usize, &[T])) {
        let index = self.load();
        for (start, values) in index.starts().chunks(from..to) {
            each(start, values);
        }
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), value| each(value));
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        let index = self.load();
        index.starts().iter(from..to).copied().try_fold(init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        self.load().starts().get(index).copied()
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let index = self.load();
        let values = index.starts();
        out.extend(indices.iter().map_while(|&i| values.get(i).copied()));
    }
}

bitview_traversable::leaf!([I: VecIndex, T] RangeMapVec<I, T> where
    T: VecValue + Copy + Formattable + Serialize + JsonSchema,
);
