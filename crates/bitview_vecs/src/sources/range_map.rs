use std::{convert::Infallible, sync::Arc};

use brk_types::Version;
use rangeindex::SharedRangeMap;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{AnyVec, Formattable, ReadableVec, TypedVec, VecIndex, VecValue};

/// Read-only vector view of a shared range map's first indexes.
/// Updates belong to the map owner; this view adds series metadata and reads.
#[derive(Clone)]
pub struct RangeMapVec<I, T> {
    name: Arc<str>,
    version: Version,
    mapping: SharedRangeMap<T, I>,
}

impl<I: VecIndex, T: VecValue> RangeMapVec<I, T> {
    pub fn new(name: &str, version: Version, mapping: SharedRangeMap<T, I>) -> Self {
        Self {
            name: Arc::from(name),
            version,
            mapping,
        }
    }

    pub fn mapping(&self) -> &SharedRangeMap<T, I> {
        &self.mapping
    }

    fn try_fold_values<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E>
    where
        T: Copy,
    {
        let index = self.mapping.load();
        index.starts().iter(from..to).copied().try_fold(init, fold)
    }
}

impl<I: VecIndex, T: VecValue> AnyVec for RangeMapVec<I, T> {
    fn version(&self) -> Version {
        self.version
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.mapping.load().len()
    }

    vecdb::any_vec_type_methods!(I, T);
}

impl<I: VecIndex, T: VecValue> TypedVec for RangeMapVec<I, T> {
    type I = I;
    type T = T;
}

impl<I: VecIndex, T: VecValue + Copy> ReadableVec<I, T> for RangeMapVec<I, T> {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        let index = self.mapping.load();
        for (_, values) in index.starts().chunks(from..to) {
            out.extend_from_slice(values);
        }
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, each: &mut dyn FnMut(usize, &[T])) {
        let index = self.mapping.load();
        for (start, values) in index.starts().chunks(from..to) {
            each(start, values);
        }
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), value| each(value));
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let Ok(acc) = self.try_fold_values(from, to, init, |acc, value| {
            Ok::<_, Infallible>(fold(acc, value))
        });
        acc
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.try_fold_values(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        self.mapping.load().starts().get(index).copied()
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let index = self.mapping.load();
        let values = index.starts();
        out.extend(indices.iter().map_while(|&i| values.get(i).copied()));
    }
}

bitview_traversable::leaf!([I: VecIndex, T] RangeMapVec<I, T> where
    T: VecValue + Copy + Formattable + Serialize + JsonSchema,
);
