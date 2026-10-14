use std::{convert::Infallible, iter::once, marker::PhantomData, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, READ_CHUNK_SIZE, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, TypedVec, VecIndex, VecValue, Version, short_type_name,
};

use vecdb::SparseRead;

use super::batches::batches;

/// Lazily combines values a fixed number of positions apart from one source.
///
/// Current and previous ranges are read separately, so a large lookback does
/// not force reading the gap between them.
pub struct LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    lookback: usize,
    compute: F,
    _marker: PhantomData<T>,
}

impl<I, S, T, F> LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    F: Fn(S, Option<S>) -> T + Clone + Send + Sync + 'static,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
        lookback: usize,
        compute: F,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            lookback,
            compute,
            _marker: PhantomData,
        }
    }

    /// Visits `[from, to)` source chunk by source chunk with the values `lookback` positions
    /// earlier (`None` before the first), one bounded batch at a time. Each batch's historical
    /// read completes before current chunks are borrowed: a source may hold a non-reentrant
    /// publication lock during its callback.
    fn try_for_each_input<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(usize, &[S], Option<&[S]>) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut previous = Vec::new();
        for (at, end) in batches(from, to.min(self.len()), self.source.cursor_chunk_size()) {
            let previous_from = at.saturating_sub(self.lookback);
            previous.clear();
            self.source.read_into_at(
                previous_from,
                end.saturating_sub(self.lookback),
                &mut previous,
            );
            let mut result = Ok(());
            self.source.for_each_chunk_at(at, end, &mut |at, current| {
                if result.is_err() {
                    return;
                }
                let prefix = self.lookback.saturating_sub(at).min(current.len());
                if prefix > 0 {
                    result = each(at, &current[..prefix], None);
                }
                let current = &current[prefix..];
                if result.is_err() || current.is_empty() {
                    return;
                }
                let at = at + prefix;
                let offset = at - self.lookback - previous_from;
                result = each(at, current, Some(&previous[offset..offset + current.len()]));
            });
            result?;
        }
        Ok(())
    }

    fn for_each_input(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(usize, &[S], Option<&[S]>),
    ) {
        let Ok(()) = self.try_for_each_input(from, to, |at, current, previous| {
            each(at, current, previous);
            Ok::<_, Infallible>(())
        });
    }

    fn values<'a>(
        &'a self,
        current: &'a [S],
        previous: Option<&'a [S]>,
    ) -> impl Iterator<Item = T> + 'a {
        current.iter().enumerate().map(move |(offset, current)| {
            (self.compute)(
                current.clone(),
                previous.map(|previous| previous[offset].clone()),
            )
        })
    }
}

impl<I, S, T, F> Clone for LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            lookback: self.lookback,
            compute: self.compute.clone(),
            _marker: PhantomData,
        }
    }
}

impl<I, S, T, F> AnyVec for LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    F: Fn(S, Option<S>) -> T + Clone + Send + Sync + 'static,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<T>()
    }
}

impl<I, S, T, F> TypedVec for LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    F: Fn(S, Option<S>) -> T + Clone + Send + Sync + 'static,
{
    type I = I;
    type T = T;
}

impl<I, S, T, F> ReadableVec<I, T> for LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
    F: Fn(S, Option<S>) -> T + Clone + Send + Sync + 'static,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |_, current, previous| {
            buf.extend(self.values(current, previous));
        });
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let chunk_size = self.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        let mut output = Vec::new();
        self.for_each_input(from, to, |at, current, previous| {
            for (chunk, current) in current.chunks(chunk_size).enumerate() {
                let offset = chunk * chunk_size;
                let previous = previous.map(|values| &values[offset..offset + current.len()]);
                output.clear();
                output.extend(self.values(current, previous));
                f(at + offset, &output);
            }
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_input(from, to, |_, current, previous| {
            self.values(current, previous).for_each(&mut *f);
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
        self.try_for_each_input(from, to, |_, current, previous| {
            accumulator = Some(
                self.values(current, previous)
                    .try_fold(accumulator.take().unwrap(), &mut fold)?,
            );
            Ok(())
        })?;
        Ok(accumulator.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let current = self.source.collect_one_at(index)?;
        let previous = index
            .checked_sub(self.lookback)
            .and_then(|index| self.source.collect_one_at(index));
        Some((self.compute)(current, previous))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        match indices {
            [] => return,
            &[index] => {
                if let Some(value) = self.collect_one_at(index) {
                    out.push(value);
                }
                return;
            }
            _ => {}
        }

        let indices = &indices[..indices.partition_point(|&index| index < self.len())];
        let source = SparseRead::new(&*self.source, indices, |index| {
            index.checked_sub(self.lookback)
        });

        out.reserve(indices.len());
        for output in 0..indices.len() {
            let current = source.current(output);
            let previous = source.previous(output);
            out.push((self.compute)(current, previous));
        }
    }
}

impl<I, S, T, F> Traversable for LazyLookbackVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue + Formattable + Serialize + JsonSchema,
    F: Fn(S, Option<S>) -> T + Clone + Send + Sync + 'static,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
