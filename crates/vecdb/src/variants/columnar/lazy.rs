use std::{marker::PhantomData, sync::Arc};

use crate::{
    AnyVec, ReadOnlyClone, ReadableVec, TypedVec, UnaryTransform, VecValue, Version,
    short_type_name,
};

use super::{ColumnId, ReadableColumnarVec};

/// Lazy scalar transformation that preserves the source's column structure.
pub struct LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: S,
    compute: fn(S::T) -> T,
    columns: PhantomData<C>,
}

impl<S, T, C> Clone for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            compute: self.compute,
            columns: PhantomData,
        }
    }
}

impl<S, T, C> LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    /// Creates a single-source lazy scalar transformation while preserving columns.
    pub fn transformed<F>(name: &str, version: Version, source: S) -> Self
    where
        F: UnaryTransform<S::T, T>,
    {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            compute: F::apply,
            columns: PhantomData,
        }
    }
}

impl<S, T, C> AnyVec for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version.combine(self.source.version())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        self.source.index_type_to_string()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<C::Row<T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<C::Row<T>>()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}

impl<S, T, C> TypedVec for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    type I = S::I;
    type T = C::Row<T>;
}

impl<S, T, C> ReadableColumnarVec<C> for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    type I = S::I;
    type T = T;

    fn for_each_column_chunk_at<F>(&self, columns: &[C], from: usize, to: usize, f: &mut F)
    where
        F: FnMut(C, usize, &[T]),
    {
        let compute = self.compute;
        let mut transformed = Vec::new();
        self.source.for_each_column_chunk_at(
            columns,
            from,
            to,
            &mut |column, row_start, values| {
                transformed.clear();
                transformed.extend(values.iter().cloned().map(compute));
                f(column, row_start, &transformed);
            },
        );
    }
}

impl<S, T, C> ReadableVec<S::I, C::Row<T>> for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<C::Row<T>>) {
        let from = from.min(self.len());
        let to = to.min(self.len());
        if from >= to {
            return;
        }

        let compute = self.compute;
        out.reserve(to - from);
        self.source
            .for_each_range_dyn_at(from, to, &mut |row| out.push(C::map(row, compute)));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(C::Row<T>)) {
        let compute = self.compute;
        self.source
            .for_each_range_dyn_at(from, to, &mut |row| f(C::map(row, compute)));
    }

    fn fold_range_at<B, F: FnMut(B, C::Row<T>) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let compute = self.compute;
        self.source
            .fold_range_at(from, to, init, |acc, row| f(acc, C::map(row, compute)))
    }

    fn try_fold_range_at<B, E, F: FnMut(B, C::Row<T>) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> std::result::Result<B, E> {
        let compute = self.compute;
        self.source
            .try_fold_range_at(from, to, init, |acc, row| f(acc, C::map(row, compute)))
    }
}

impl<S, T, C> ReadOnlyClone for LazyColumnarVec<S, T, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    T: VecValue,
{
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}
