use std::{convert::Infallible, iter};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use rangeindex::SharedRangeMap;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, ReadableBoxedVec, ReadableCloneableVec, ReadableVec,
    TypedVec, VecIndex, VecValue, Version,
};

/// Reverse lookups through resident boundaries, bounded by the source domain.
/// The source supplies metadata only; its values are never read. The map must
/// cover every source index and share the source's publication/update protocol.
#[derive(Clone)]
pub struct RangeMapLookupVec<I: VecIndex, T: VecIndex + VecValue> {
    mapping: SharedRangeMap<I, T>,
    source: ReadableBoxedVec<I, T>,
}

impl<I: VecIndex, T: VecIndex + VecValue> RangeMapLookupVec<I, T> {
    pub(crate) fn new(
        mapping: &SharedRangeMap<I, T>,
        source: &(impl ReadableCloneableVec<I, T> + ?Sized),
    ) -> Self {
        Self {
            mapping: mapping.clone(),
            source: source.read_only_boxed_clone(),
        }
    }

    fn try_fold_indices<B, E>(
        &self,
        indices: impl IntoIterator<Item = usize>,
        init: B,
        mut fold: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let len = self.visible_len();
        let mapping = self.mapping.read();
        let mut cursor = mapping.cursor();
        indices
            .into_iter()
            .take_while(|&index| index < len)
            .try_fold(init, |acc, index| {
                let value = cursor
                    .get(I::from(index))
                    .expect("range map must cover every source index");
                fold(acc, value)
            })
    }
}

impl<I: VecIndex, T: VecIndex + VecValue> AnyVec for RangeMapLookupVec<I, T> {
    fn version(&self) -> Version {
        self.source.version()
    }

    fn name(&self) -> &str {
        self.source.name()
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn visible_len(&self) -> usize {
        self.source.visible_len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.source.index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        self.source.value_type_to_size_of()
    }

    fn value_type_to_string(&self) -> &'static str {
        self.source.value_type_to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}

impl<I: VecIndex, T: VecIndex + VecValue> TypedVec for RangeMapLookupVec<I, T> {
    type I = I;
    type T = T;
}

impl<I: VecIndex, T: VecIndex + VecValue> ReadableVec<I, T> for RangeMapLookupVec<I, T> {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        out.reserve(to.min(self.visible_len()).saturating_sub(from));
        self.for_each_range_dyn_at(from, to, &mut |value| out.push(value));
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
        match self.try_fold_indices(from..to, init, |acc, value| {
            Ok::<_, Infallible>(fold(acc, value))
        }) {
            Ok(acc) => acc,
            Err(error) => match error {},
        }
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.try_fold_indices(from..to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.visible_len() {
            return None;
        }
        self.mapping.read().get_shared(I::from(index))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        out.reserve(indices.len());
        self.try_fold_indices(indices.iter().copied(), (), |(), value| {
            out.push(value);
            Ok::<_, Infallible>(())
        })
        .unwrap();
    }
}

impl<I: VecIndex, T> Traversable for RangeMapLookupVec<I, T>
where
    T: VecIndex + VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}
