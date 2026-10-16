use std::{convert::Infallible, marker::PhantomData, sync::Arc};

use brk_types::Height;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyVec, BinaryTransform, CheckedSub, Formattable, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, SparseRead, TypedVec, VecIndex, VecValue, Version, WindowBatch, WindowBuffers,
    before_window, before_window_bounds, read_batches,
};

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

    /// Both window totals, current minus the cumulative value before the window, combined by `F`.
    #[inline(always)]
    fn compute(source: S, source_before: S, operand: C, operand_before: C) -> T {
        F::apply(
            source.checked_sub(source_before).unwrap_or_default(),
            operand.checked_sub(operand_before).unwrap_or_default(),
        )
    }

    /// Visits `[from, to)` one bounded batch at a time with its window starts and both inputs'
    /// batches ([`WindowBuffers`]).
    fn try_for_each_batch<E>(
        &self,
        from: usize,
        to: usize,
        mut visit: impl FnMut(&[Height], WindowBatch<'_, S>, WindowBatch<'_, C>) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut starts = Vec::new();
        let (mut source, mut operand) = (WindowBuffers::default(), WindowBuffers::default());
        for (at, end) in read_batches(from, to.min(self.len()), self.source.cursor_chunk_size()) {
            starts.clear();
            self.window_starts.read_into_at(at, end, &mut starts);
            let bounds = before_window_bounds(&starts);
            visit(
                &starts,
                source.read(&*self.source, at, end, bounds),
                operand.read(&*self.operand, at, end, bounds),
            )?;
        }
        Ok(())
    }

    fn values<'a>(
        starts: &'a [Height],
        source: WindowBatch<'a, S>,
        operand: WindowBatch<'a, C>,
    ) -> impl Iterator<Item = T> + 'a {
        source.current.iter().zip(operand.current).zip(starts).map(
            move |((current, operand_current), start)| {
                let start = start.to_usize();
                Self::compute(
                    current.clone(),
                    source.before(start),
                    operand_current.clone(),
                    operand.before(start),
                )
            },
        )
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

    fn is_mutable(&self) -> bool {
        self.source.is_mutable() || self.operand.is_mutable() || self.window_starts.is_mutable()
    }

    vecdb::any_vec_type_methods!(Height, T);
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

        let previous = before_window(self.window_starts.collect_one_at(index)?.to_usize());
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
            before_window(starts.next().unwrap().to_usize())
        });

        let mut starts = window_starts.iter();
        let operand = SparseRead::new(&*self.operand, indices, |_| {
            before_window(starts.next().unwrap().to_usize())
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

bitview_traversable::leaf!([S, C, T, F] LazyRollingRatioVec<S, C, T, F> where
    S: VecValue + CheckedSub + Default,
    C: VecValue + CheckedSub + Default,
    T: VecValue + Formattable + Serialize + JsonSchema,
    F: BinaryTransform<S, C, T> + Send + Sync,
);
