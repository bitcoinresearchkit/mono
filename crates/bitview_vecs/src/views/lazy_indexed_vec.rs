use std::{iter::once, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, READ_CHUNK_SIZE, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, TypedVec, VecIndex, VecValue, Version, short_type_name,
};

/// One captured operation, dispatched once for each scalar read or whole chunk.
trait IndexedTransform<I, S, M, T>: Send + Sync {
    fn apply(&self, index: I, source: S, metadata: M) -> T;
    fn append(&self, at: usize, source: &[S], metadata: &[M], out: &mut Vec<T>);
}

impl<I, S, M, T, F> IndexedTransform<I, S, M, T> for F
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    F: Fn(I, S, M) -> T + Send + Sync,
{
    fn apply(&self, index: I, source: S, metadata: M) -> T {
        self(index, source, metadata)
    }

    fn append(&self, at: usize, source: &[S], metadata: &[M], out: &mut Vec<T>) {
        // Validate the end once rather than checked addition per element.
        let indices = at..at + source.len();
        out.extend(
            indices
                .zip(source.iter().cloned())
                .zip(metadata.iter().cloned())
                .map(|((index, source), metadata)| self(I::from(index), source, metadata)),
        );
    }
}

/// Lazily transforms one metric source with aligned index metadata.
/// Bounded metadata reads preserve its owner\'s cold-cache admission policy.
pub struct LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    metadata: ReadableBoxedVec<I, M>,
    compute: Arc<dyn IndexedTransform<I, S, M, T>>,
}

impl<I, S, M, T> LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
        metadata: &(impl ReadableCloneableVec<I, M> + ?Sized),
        compute: impl Fn(I, S, M) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            metadata: metadata.read_only_boxed_clone(),
            compute: Arc::new(compute),
        }
    }

    fn try_fold_values<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let to = to.min(self.len());
        if from >= to {
            return Ok(init);
        }

        let metadata = self.metadata.collect_range_dyn(from, to);
        let source = self.source.collect_range_dyn(from, to);
        source.into_iter().zip(metadata).enumerate().try_fold(
            init,
            |accumulator, (offset, (source, metadata))| {
                fold(
                    accumulator,
                    self.compute.apply(I::from(from + offset), source, metadata),
                )
            },
        )
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        self.for_each_chunk_at(from, to, &mut |_, values| {
            values.iter().cloned().for_each(&mut each);
        });
    }
}

impl<I, S, M, T> Clone for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            metadata: self.metadata.clone(),
            compute: Arc::clone(&self.compute),
        }
    }
}

impl<I, S, M, T> AnyVec for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version() + self.metadata.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        self.source.len().min(self.metadata.len())
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}

impl<I, S, M, T> TypedVec for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    type I = I;
    type T = T;
}

impl<I, S, M, T> ReadableVec<I, T> for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }
        let metadata = self.metadata.collect_range_dyn(from, to);
        let to = from + metadata.len();
        buf.reserve(to.saturating_sub(from));
        self.source.for_each_chunk_at(from, to, &mut |at, values| {
            self.compute.append(
                at,
                values,
                &metadata[at - from..at - from + values.len()],
                buf,
            );
        });
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }
        let metadata = self.metadata.collect_range_dyn(from, to);
        let to = from + metadata.len();
        let mut output = Vec::new();
        let chunk_size = self.source.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        self.source.for_each_chunk_at(from, to, &mut |at, values| {
            let mut at = at;
            for values in values.chunks(chunk_size) {
                output.clear();
                self.compute.append(
                    at,
                    values,
                    &metadata[at - from..at - from + values.len()],
                    &mut output,
                );
                f(at, &output);
                at += values.len();
            }
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_value(from, to, f);
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_chunk_at(from, to, &mut |_, values| {
            acc = Some(values.iter().cloned().fold(acc.take().unwrap(), &mut fold));
        });
        acc.unwrap()
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
        let source = self.source.collect_one_at(index)?;
        let metadata = self.metadata.collect_one_at(index)?;
        Some(self.compute.apply(I::from(index), source, metadata))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let len = self.len();
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        let source = self.source.read_sorted_at(indices);
        let metadata = self.metadata.read_sorted_at(indices);

        out.reserve(source.len());
        for ((&index, value), metadata) in indices.iter().zip(source).zip(metadata) {
            out.push(self.compute.apply(I::from(index), value, metadata));
        }
    }
}

impl<I, S, M, T> Traversable for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}
