use std::{convert::Infallible, iter, marker::PhantomData, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::Height;
use input::RollingInput;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, BinaryTransform, CheckedSub, Formattable, PrintableIndex,
    READ_CHUNK_SIZE, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, SparseRead, TypedVec,
    VecIndex, VecValue, Version, short_type_name,
};

mod input;

/// Rolling transform from a cumulative source, an aligned operand, and window starts.
/// Snapshots belong to the input readers; this view retains no full-height result.
pub struct LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue,
    C: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<Height, S>,
    operand: ReadableBoxedVec<Height, C>,
    operand_transform: fn(Height, C) -> C,
    window_starts: ReadableBoxedVec<Height, Height>,
    _marker: PhantomData<(T, F)>,
}

impl<S, C, T, F> LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue,
    F: BinaryTransform<S, C, T>,
{
    #[inline(always)]
    fn identity_operand(_: Height, value: C) -> C {
        value
    }

    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + ?Sized),
        operand: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        window_starts: &(impl ReadableCloneableVec<Height, Height> + ?Sized),
    ) -> Self {
        Self::with_operand_transform(
            name,
            version,
            source,
            operand,
            window_starts,
            Self::identity_operand,
        )
    }

    fn with_operand_transform(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + ?Sized),
        operand: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        window_starts: &(impl ReadableCloneableVec<Height, Height> + ?Sized),
        operand_transform: fn(Height, C) -> C,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            operand: operand.read_only_boxed_clone(),
            operand_transform,
            window_starts: window_starts.read_only_boxed_clone(),
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    fn previous_index(start: Height) -> Option<usize> {
        start.to_usize().checked_sub(1)
    }

    #[inline(always)]
    fn compute(
        &self,
        index: usize,
        previous: Option<usize>,
        source_current: S,
        source_previous: S,
        operand_current: C,
        operand_previous: C,
    ) -> T {
        F::apply(
            source_current
                .checked_sub(source_previous)
                .unwrap_or_default(),
            (self.operand_transform)(Height::from(index), operand_current)
                .checked_sub(
                    previous
                        .map(|index| {
                            (self.operand_transform)(Height::from(index), operand_previous)
                        })
                        .unwrap_or_default(),
                )
                .unwrap_or_default(),
        )
    }

    fn try_fold_values<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let mut accumulator = Some(Ok(init));
        self.for_each_input(from, to, |at, source, operand, starts| {
            accumulator = Some(accumulator.take().unwrap().and_then(|accumulator| {
                self.transformed_values(at, source, operand, starts)
                    .try_fold(accumulator, &mut fold)
            }));
        });
        accumulator.unwrap()
    }

    fn for_each_input(
        &self,
        from: usize,
        to: usize,
        mut visit: impl FnMut(usize, &RollingInput<S>, &RollingInput<C>, &[Height]),
    ) {
        let to = to
            .min(self.source.len())
            .min(self.operand.len())
            .min(self.window_starts.len());
        if from >= to {
            return;
        }
        let starts = self.window_starts.collect_range_dyn(from, to);
        let starts = starts.as_slice();
        let source = RollingInput::new(&self.source, from, to, starts);
        let operand = RollingInput::new(&self.operand, from, to, starts);
        visit(from, &source, &operand, starts);
    }

    fn transformed_values<'a>(
        &'a self,
        at: usize,
        source: &'a RollingInput<S>,
        operand: &'a RollingInput<C>,
        starts: &'a [Height],
    ) -> impl Iterator<Item = T> + 'a {
        source
            .current()
            .iter()
            .zip(operand.current())
            .zip(starts)
            .enumerate()
            .map(move |(offset, ((current, operand_current), start))| {
                let index = at + offset;
                let prior = Self::previous_index(*start);
                self.compute(
                    index,
                    prior,
                    current.clone(),
                    prior.map(|i| source.previous(i)).unwrap_or_default(),
                    operand_current.clone(),
                    prior.map(|i| operand.previous(i)).unwrap_or_default(),
                )
            })
    }
}

impl<S, C, T, F> Clone for LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue,
    C: VecValue,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            operand: self.operand.clone(),
            operand_transform: self.operand_transform,
            window_starts: self.window_starts.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, C, T, F> AnyVec for LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue,
    F: BinaryTransform<S, C, T> + Send + Sync,
{
    fn version(&self) -> Version {
        self.base_version
            + self.source.version()
            + self.operand.version()
            + self.window_starts.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source
            .len()
            .min(self.operand.len())
            .min(self.window_starts.len())
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
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

impl<S, C, T, F> TypedVec for LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue,
    F: BinaryTransform<S, C, T> + Send + Sync,
{
    type I = Height;
    type T = T;
}

impl<S, C, T, F> ReadableVec<Height, T> for LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue,
    F: BinaryTransform<S, C, T> + Send + Sync,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |at, source, operand, starts| {
            buf.extend(self.transformed_values(at, source, operand, starts));
        });
    }

    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, each: &mut dyn FnMut(usize, &[T])) {
        let size = self.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        let mut output = Vec::new();
        let mut at = from;
        self.for_each_input(from, to, |from, source, operand, starts| {
            for value in self.transformed_values(from, source, operand, starts) {
                output.push(value);
                if output.len() == size {
                    each(at, &output);
                    at += output.len();
                    output.clear();
                }
            }
        });
        if !output.is_empty() {
            each(at, &output);
        }
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.for_each_chunk_at(from, to, &mut |_, values| {
            values.iter().cloned().for_each(&mut *each)
        });
    }

    fn fold_range_at<B, G: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: G,
    ) -> B {
        self.try_fold_values(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(fold(accumulator, value))
        })
        .unwrap()
    }

    fn try_fold_range_at<B, E, G: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: G,
    ) -> Result<B, E> {
        self.try_fold_values(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let previous = Self::previous_index(self.window_starts.collect_one_at(index)?);
        Some(
            self.compute(
                index,
                previous,
                self.source.collect_one_at(index)?,
                previous
                    .and_then(|index| self.source.collect_one_at(index))
                    .unwrap_or_default(),
                self.operand.collect_one_at(index)?,
                previous
                    .and_then(|index| self.operand.collect_one_at(index))
                    .unwrap_or_default(),
            ),
        )
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
        if indices.is_empty() {
            return;
        }

        let window_starts = self.window_starts.read_sorted_at(indices);
        let mut starts = window_starts.iter();
        let source = SparseRead::new(&*self.source, indices, |_| {
            Self::previous_index(*starts.next().unwrap())
        });

        let mut starts = window_starts.iter();
        let operand = SparseRead::new(&*self.operand, indices, |_| {
            Self::previous_index(*starts.next().unwrap())
        });

        out.reserve(indices.len());
        for (output, &index) in indices.iter().enumerate() {
            let previous = Self::previous_index(window_starts[output]);
            out.push(self.compute(
                index,
                previous,
                source.current(output),
                source.previous(output).unwrap_or_default(),
                operand.current(output),
                operand.previous(output).unwrap_or_default(),
            ));
        }
    }
}

impl<S, C, T, F> Traversable for LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue + Formattable + Serialize + JsonSchema,
    F: BinaryTransform<S, C, T> + Send + Sync,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
