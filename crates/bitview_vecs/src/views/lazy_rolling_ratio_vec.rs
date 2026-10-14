use std::{convert::Infallible, iter, marker::PhantomData, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::Height;
use input::RollingInput;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, BinaryTransform, CheckedSub, Formattable, PrintableIndex,
    ReadableBoxedVec, ReadableCloneableVec, ReadableVec, SparseRead, TypedVec, VecIndex, VecValue,
    Version, short_type_name,
};

use super::batches::batches;

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
    window_starts: ReadableBoxedVec<Height, Height>,
    _marker: PhantomData<(T, F)>,
}

impl<S, C, T, F> LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue,
    F: BinaryTransform<S, C, T> + Send + Sync,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + ?Sized),
        operand: &(impl ReadableCloneableVec<Height, C> + ?Sized),
        window_starts: &(impl ReadableCloneableVec<Height, Height> + ?Sized),
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            operand: operand.read_only_boxed_clone(),
            window_starts: window_starts.read_only_boxed_clone(),
            _marker: PhantomData,
        }
    }

    /// The cumulative value preceding a window: none for a window starting at zero.
    #[inline(always)]
    fn previous_index(start: &Height) -> Option<usize> {
        start.to_usize().checked_sub(1)
    }

    /// Both window totals, current minus preceding cumulative value, combined by `F`.
    #[inline(always)]
    fn compute(source: S, source_previous: S, operand: C, operand_previous: C) -> T {
        F::apply(
            source.checked_sub(source_previous).unwrap_or_default(),
            operand.checked_sub(operand_previous).unwrap_or_default(),
        )
    }

    /// Visits `[from, to)` one bounded batch at a time with its window starts and both inputs'
    /// current and preceding values.
    fn try_for_each_batch<E>(
        &self,
        from: usize,
        to: usize,
        mut visit: impl FnMut(&[Height], &RollingInput<S>, &RollingInput<C>) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut starts = Vec::new();
        let (mut source, mut operand) = (RollingInput::default(), RollingInput::default());
        for (at, end) in batches(from, to.min(self.len()), self.source.cursor_chunk_size()) {
            starts.clear();
            self.window_starts.read_into_at(at, end, &mut starts);
            let bounds = starts.iter().filter_map(Self::previous_index).fold(
                None,
                |bounds: Option<(usize, usize)>, index| {
                    Some(bounds.map_or((index, index), |(low, high)| {
                        (low.min(index), high.max(index))
                    }))
                },
            );
            source.read(&self.source, at, end, bounds);
            operand.read(&self.operand, at, end, bounds);
            visit(&starts, &source, &operand)?;
        }
        Ok(())
    }

    fn values<'a>(
        starts: &'a [Height],
        source: &'a RollingInput<S>,
        operand: &'a RollingInput<C>,
    ) -> impl Iterator<Item = T> + 'a {
        source
            .current()
            .iter()
            .zip(operand.current())
            .zip(starts)
            .map(move |((current, operand_current), start)| {
                let previous = Self::previous_index(start);
                Self::compute(
                    current.clone(),
                    previous.map(|i| source.previous(i)).unwrap_or_default(),
                    operand_current.clone(),
                    previous.map(|i| operand.previous(i)).unwrap_or_default(),
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
        let Ok(()) = self.try_for_each_batch(from, to, |starts, source, operand| {
            buf.extend(Self::values(starts, source, operand));
            Ok::<_, Infallible>(())
        });
    }

    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, each: &mut dyn FnMut(usize, &[T])) {
        let mut output = Vec::new();
        let mut at = from;
        let Ok(()) = self.try_for_each_batch(from, to, |starts, source, operand| {
            output.clear();
            output.extend(Self::values(starts, source, operand));
            each(at, &output);
            at += output.len();
            Ok::<_, Infallible>(())
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        let Ok(()) = self.try_for_each_batch(from, to, |starts, source, operand| {
            Self::values(starts, source, operand).for_each(&mut *each);
            Ok::<_, Infallible>(())
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
        self.try_for_each_batch(from, to, |starts, source, operand| {
            accumulator = Some(
                Self::values(starts, source, operand)
                    .try_fold(accumulator.take().unwrap(), &mut fold)?,
            );
            Ok(())
        })?;
        Ok(accumulator.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let previous = Self::previous_index(&self.window_starts.collect_one_at(index)?);
        Some(Self::compute(
            self.source.collect_one_at(index)?,
            previous
                .and_then(|index| self.source.collect_one_at(index))
                .unwrap_or_default(),
            self.operand.collect_one_at(index)?,
            previous
                .and_then(|index| self.operand.collect_one_at(index))
                .unwrap_or_default(),
        ))
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
            Self::previous_index(starts.next().unwrap())
        });

        let mut starts = window_starts.iter();
        let operand = SparseRead::new(&*self.operand, indices, |_| {
            Self::previous_index(starts.next().unwrap())
        });

        out.reserve(indices.len());
        for output in 0..indices.len() {
            out.push(Self::compute(
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
