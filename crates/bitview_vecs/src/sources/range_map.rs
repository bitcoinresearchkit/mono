use std::{iter, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::Version;
use rangeindex::SharedRangeMap;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, ReadableVec, TypedVec, VecIndex, VecValue,
    short_type_name,
};

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

    fn with_range<R>(&self, from: usize, to: usize, read: impl FnOnce(&[T]) -> R) -> R {
        let index = self.mapping.read();
        let values = index.as_slice();
        let to = to.min(values.len());
        read(&values[from.min(to)..to])
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
        self.mapping.read().len()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }
}

impl<I: VecIndex, T: VecValue> TypedVec for RangeMapVec<I, T> {
    type I = I;
    type T = T;
}

impl<I: VecIndex, T: VecValue + Copy> ReadableVec<I, T> for RangeMapVec<I, T> {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        self.with_range(from, to, |values| out.extend_from_slice(values));
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, each: &mut dyn FnMut(usize, &[T])) {
        self.with_range(from, to, |values| {
            if !values.is_empty() {
                each(from, values);
            }
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.with_range(from, to, |values| values.iter().copied().for_each(each));
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, fold: F) -> B {
        self.with_range(from, to, |values| values.iter().copied().fold(init, fold))
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.with_range(from, to, |values| {
            values.iter().copied().try_fold(init, fold)
        })
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        self.mapping.read().as_slice().get(index).copied()
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let index = self.mapping.read();
        let values = index.as_slice();
        out.extend(
            indices
                .iter()
                .copied()
                .take_while(|&i| i < values.len())
                .map(|i| values[i]),
        );
    }
}

impl<I: VecIndex, T> Traversable for RangeMapVec<I, T>
where
    T: VecValue + Copy + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}
