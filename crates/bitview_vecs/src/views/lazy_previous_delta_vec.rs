use std::{convert::Infallible, iter::once, marker::PhantomData, mem, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, CheckedSub, Cursor, Formattable, Ident, READ_CHUNK_SIZE,
    ReadableBoxedVec, ReadableCloneableVec, ReadableVec, SparseRead, TypedVec, UnaryTransform,
    VecIndex, VecValue, Version, short_type_name,
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
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
    ) -> Self {
        Self::transformed(name, version, source)
    }
}

impl<I, S, T, F> LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    pub fn transformed(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
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
    fn for_each_input(&self, from: usize, to: usize, mut each: impl FnMut(usize, &[S], &mut S)) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }
        let mut previous = if from == 0 { Some(S::default()) } else { None };
        let mut emitted = from;
        self.source
            .for_each_chunk_at(from.saturating_sub(1), to, &mut |_, mut values| {
                if previous.is_none() {
                    previous = Some(values[0].clone());
                    values = &values[1..];
                }
                if !values.is_empty() {
                    each(emitted, values, previous.as_mut().unwrap());
                    emitted += values.len();
                }
            });
    }

    fn append_delta(values: &[S], previous: &mut S, out: &mut Vec<T>) {
        let Some((first, rest)) = values.split_first() else {
            return;
        };
        out.push(F::apply(
            first
                .clone()
                .checked_sub(previous.clone())
                .unwrap_or_default(),
        ));
        out.extend(
            rest.iter()
                .cloned()
                .zip(values.iter().cloned())
                .map(|(current, previous)| {
                    F::apply(current.checked_sub(previous).unwrap_or_default())
                }),
        );
        *previous = values.last().unwrap().clone();
    }
}

impl<I, S, T, F> ReadableVec<I, T> for LazyPreviousDeltaVec<I, S, T, F>
where
    I: VecIndex,
    S: VecValue + CheckedSub + Default,
    T: VecValue,
    F: UnaryTransform<S, T>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |_, values, previous| {
            Self::append_delta(values, previous, buf);
        });
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let chunk_size = self.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        let mut output = Vec::new();
        self.for_each_input(from, to, |at, values, previous| {
            for (offset, values) in values.chunks(chunk_size).enumerate() {
                output.clear();
                Self::append_delta(values, previous, &mut output);
                f(at + offset * chunk_size, &output);
            }
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_chunk_at(from, to, &mut |_, values| {
            values.iter().cloned().for_each(&mut *f);
        });
    }

    fn fold_range_at<B, G: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: G,
    ) -> B {
        self.try_fold_range_at(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(f(accumulator, value))
        })
        .unwrap()
    }

    fn try_fold_range_at<B, E, G: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: G,
    ) -> Result<B, E> {
        let mut accumulator = Some(Ok(init));
        self.for_each_input(from, to, |_, values, previous| {
            accumulator = Some(accumulator.take().unwrap().and_then(|accumulator| {
                values
                    .iter()
                    .cloned()
                    .try_fold(accumulator, |accumulator, current| {
                        let previous = mem::replace(previous, current.clone());
                        fold(
                            accumulator,
                            F::apply(current.checked_sub(previous).unwrap_or_default()),
                        )
                    })
            }));
        });
        accumulator.unwrap()
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
        let indices = &indices[..indices.partition_point(|&i| i < len)];
        if indices.len() > 1
            && indices.windows(2).all(|pair| pair[1] == pair[0] + 1)
            && !self.source.is_mutable()
        {
            self.read_into_at(indices[0], indices[indices.len() - 1] + 1, out);
            return;
        }
        if let (Some(&first), Some(&last)) = (indices.first(), indices.last())
            && last - first >= indices.len()
            && !self.source.is_mutable()
            && let Some(values) = SparseRead::try_new(&*self.source, indices, |i| i.checked_sub(1))
        {
            out.extend((0..indices.len()).map(|slot| {
                F::apply(
                    values
                        .current(slot)
                        .checked_sub(values.previous(slot).unwrap_or_default())
                        .unwrap_or_default(),
                )
            }));
            return;
        }
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
        once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}
