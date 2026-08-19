use std::{marker::PhantomData, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, CheckedSub, Cursor, Formattable, Ident, ReadableBoxedVec,
    ReadableVec, TypedVec, UnaryTransform, VecIndex, VecValue, Version, short_type_name,
};

/// Lazy `source[index] - source[index - 1]`, with zero before the first value.
///
/// This is a single-source view: it follows source growth automatically and
/// stores nothing on disk.
pub struct LazyPreviousDeltaVec<I, S, T = S, F = Ident>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    _output: PhantomData<fn(S) -> (T, F)>,
}

impl<I, S> LazyPreviousDeltaVec<I, S>
where
    I: VecIndex,
    S: VecValue,
{
    pub fn new(name: &str, version: Version, source: ReadableBoxedVec<I, S>) -> Self {
        Self::transformed(name, version, source)
    }
}

impl<I, S, T, F> LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    pub fn transformed(name: &str, version: Version, source: ReadableBoxedVec<I, S>) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            _output: PhantomData,
        }
    }
}

impl<I, S, T, F> Clone for LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            _output: PhantomData,
        }
    }
}

impl<I, S, T, F> AnyVec for LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len()
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

impl<I, S, T, F> TypedVec for LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    type I = I;
    type T = T;
}

impl<I, S, T, F> LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue + CheckedSub + Default,
    T: VecValue,
    F: UnaryTransform<S, T>,
{
    fn for_each_delta(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }

        let read_from = from.saturating_sub(1);
        let values = self.source.collect_range_dyn(read_from, to);
        let mut values = values.into_iter();
        let mut previous = if from == 0 {
            S::default()
        } else {
            values.next().unwrap()
        };

        for current in values {
            each(F::apply(
                current.clone().checked_sub(previous).unwrap_or_default(),
            ));
            previous = current;
        }
    }
}

impl<I, S, T, F> ReadableVec<I, T> for LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue + CheckedSub + Default,
    T: VecValue,
    F: UnaryTransform<S, T>,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_delta(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_delta(from, to, f);
    }

    fn fold_range_at<B, G: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: G) -> B {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().fold(init, f)
    }

    fn try_fold_range_at<B, E, G: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: G,
    ) -> Result<B, E> {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().try_fold(init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let current = self.source.collect_one_at(index)?;
        let previous = index
            .checked_sub(1)
            .and_then(|index| self.source.collect_one_at(index))
            .unwrap_or_default();
        Some(F::apply(current.checked_sub(previous).unwrap_or_default()))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let len = self.len();
        let mut source = Cursor::new(&*self.source);
        let mut cached: Option<(usize, T)> = None;

        out.reserve(indices.len());
        for &index in indices {
            if index >= len {
                break;
            }
            if let Some((cached_index, ref value)) = cached
                && cached_index == index
            {
                out.push(value.clone());
                continue;
            }

            let previous = index
                .checked_sub(1)
                .and_then(|index| source.get(index))
                .unwrap_or_default();
            let Some(current) = source.get(index) else {
                continue;
            };
            let value = F::apply(current.checked_sub(previous).unwrap_or_default());
            cached = Some((index, value.clone()));
            out.push(value);
        }
    }
}

impl<I, S, T, F> Traversable for LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue + CheckedSub + Default,
    T: VecValue + Formattable + Serialize + JsonSchema,
    F: UnaryTransform<S, T>,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}
