use std::{convert::Infallible, marker::PhantomData, sync::Arc};

use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyVec, Formattable, READ_CHUNK_SIZE, ReadableBoxedVec, ReadableCloneableVec, ReadableVec,
    TypedVec, VecIndex, VecValue, Version, read_batches,
};

/// Lazily transforms one metric source with aligned index metadata.
/// Bounded metadata reads preserve its owner's cold-cache admission policy.
#[derive(Clone)]
pub struct LazyIndexedVec<I, S, M, T, F>
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
    compute: F,
    _marker: PhantomData<T>,
}

impl<I, S, M, T, F> LazyIndexedVec<I, S, M, T, F>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
    F: Fn(I, S, M) -> T + Clone + Send + Sync + 'static,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
        metadata: &(impl ReadableCloneableVec<I, M> + ?Sized),
        compute: F,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            metadata: metadata.read_only_boxed_clone(),
            compute,
            _marker: PhantomData,
        }
    }

    /// Visits `[from, to)` source chunk by source chunk with the aligned metadata, one bounded
    /// batch at a time. Each batch's metadata read completes before source chunks are borrowed:
    /// a source may hold a non-reentrant publication lock during its callback.
    fn try_for_each_input<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(usize, &[S], &[M]) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut metadata = Vec::new();
        for (at, end) in read_batches(from, to.min(self.len()), self.source.cursor_chunk_size()) {
            metadata.clear();
            self.metadata.read_into_at(at, end, &mut metadata);
            let mut result = Ok(());
            self.source
                .for_each_chunk_at(at, at + metadata.len(), &mut |chunk_at, values| {
                    if result.is_ok() {
                        let offset = chunk_at - at;
                        result = each(chunk_at, values, &metadata[offset..offset + values.len()]);
                    }
                });
            result?;
            if metadata.len() < end - at {
                break;
            }
        }
        Ok(())
    }

    fn for_each_input(&self, from: usize, to: usize, mut each: impl FnMut(usize, &[S], &[M])) {
        let Ok(()) = self.try_for_each_input(from, to, |at, source, metadata| {
            each(at, source, metadata);
            Ok::<_, Infallible>(())
        });
    }

    fn values<'a>(
        &'a self,
        at: usize,
        source: &'a [S],
        metadata: &'a [M],
    ) -> impl Iterator<Item = T> + 'a {
        (at..)
            .zip(source)
            .zip(metadata)
            .map(move |((index, source), metadata)| {
                (self.compute)(I::from(index), source.clone(), metadata.clone())
            })
    }
}

impl<I, S, M, T, F> AnyVec for LazyIndexedVec<I, S, M, T, F>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
    F: Fn(I, S, M) -> T + Clone + Send + Sync + 'static,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version() + self.metadata.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len().min(self.metadata.len())
    }

    /// A view of a mutable source changes with it.
    fn is_mutable(&self) -> bool {
        self.source.is_mutable() || self.metadata.is_mutable()
    }

    vecdb::any_vec_type_methods!(I, T);
}

impl<I, S, M, T, F> TypedVec for LazyIndexedVec<I, S, M, T, F>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
    F: Fn(I, S, M) -> T + Clone + Send + Sync + 'static,
{
    type I = I;
    type T = T;
}

impl<I, S, M, T, F> ReadableVec<I, T> for LazyIndexedVec<I, S, M, T, F>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
    F: Fn(I, S, M) -> T + Clone + Send + Sync + 'static,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |at, source, metadata| {
            buf.extend(self.values(at, source, metadata));
        });
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let mut output = Vec::new();
        let chunk_size = self.source.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        self.for_each_input(from, to, |at, source, metadata| {
            for (offset, (source, metadata)) in (0..)
                .step_by(chunk_size)
                .zip(source.chunks(chunk_size).zip(metadata.chunks(chunk_size)))
            {
                output.clear();
                output.extend(self.values(at + offset, source, metadata));
                f(at + offset, &output);
            }
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_input(from, to, |at, source, metadata| {
            self.values(at, source, metadata).for_each(&mut *f);
        });
    }

    fn fold_range_at<B, G: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: G,
    ) -> B {
        let Ok(accumulator) = self.try_fold_range_at(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(fold(accumulator, value))
        });
        accumulator
    }

    fn try_fold_range_at<B, E, G: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: G,
    ) -> Result<B, E> {
        let mut accumulator = Some(init);
        self.try_for_each_input(from, to, |at, source, metadata| {
            accumulator = Some(
                self.values(at, source, metadata)
                    .try_fold(accumulator.take().unwrap(), &mut fold)?,
            );
            Ok(())
        })?;
        Ok(accumulator.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let source = self.source.collect_one_at(index)?;
        let metadata = self.metadata.collect_one_at(index)?;
        Some((self.compute)(I::from(index), source, metadata))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let len = self.len();
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        let source = self.source.read_sorted_at(indices);
        let metadata = self.metadata.read_sorted_at(indices);

        out.reserve(source.len());
        for ((&index, value), metadata) in indices.iter().zip(source).zip(metadata) {
            out.push((self.compute)(I::from(index), value, metadata));
        }
    }
}

bitview_traversable::leaf!([I, S, M, T, F] LazyIndexedVec<I, S, M, T, F> where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue + Formattable + Serialize + JsonSchema,
    F: Fn(I, S, M) -> T + Clone + Send + Sync + 'static,
);
