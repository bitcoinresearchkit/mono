use std::{iter::once, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, READ_CHUNK_SIZE, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, TypedVec, VecIndex, VecValue, Version, short_type_name,
};

use vecdb::SparseRead;

trait LookbackTransform<S, T>: Send + Sync {
    fn apply(&self, current: S, previous: Option<S>) -> T;
    fn append(&self, current: &[S], previous: Option<&[S]>, out: &mut Vec<T>);
}

impl<S: VecValue, T, F: Fn(S, Option<S>) -> T + Send + Sync> LookbackTransform<S, T> for F {
    fn apply(&self, current: S, previous: Option<S>) -> T {
        self(current, previous)
    }
    fn append(&self, current: &[S], previous: Option<&[S]>, out: &mut Vec<T>) {
        match previous {
            Some(previous) => out.extend(
                current
                    .iter()
                    .cloned()
                    .zip(previous.iter().cloned())
                    .map(|(current, previous)| self(current, Some(previous))),
            ),
            None => out.extend(current.iter().cloned().map(|current| self(current, None))),
        }
    }
}

/// Lazily combines values a fixed number of positions apart from one source.
///
/// Current and previous ranges are read separately, so a large lookback does
/// not force reading the gap between them.
pub struct LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    lookback: usize,
    compute: Arc<dyn LookbackTransform<S, T>>,
}

impl<I, S, T> LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
        lookback: usize,
        compute: impl Fn(S, Option<S>) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            lookback,
            compute: Arc::new(compute),
        }
    }
}

impl<I, S, T> LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    fn try_fold_lookback<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let mut accumulator = init;
        let to = to.min(self.len());
        if from >= to {
            return Ok(accumulator);
        }

        let previous_from = from.saturating_sub(self.lookback);
        let previous_to = to.saturating_sub(self.lookback);
        let previous = self.source.collect_range_dyn(previous_from, previous_to);
        let current = self.source.collect_range_dyn(from, to);

        for (offset, current) in current.into_iter().enumerate() {
            let index = from + offset;
            let previous = index
                .checked_sub(self.lookback)
                .map(|index| previous[index - previous_from].clone());
            accumulator = fold(accumulator, self.compute.apply(current, previous))?;
        }
        Ok(accumulator)
    }

    fn for_each_input(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(usize, &[S], Option<&[S]>),
    ) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }
        let previous_from = from.saturating_sub(self.lookback);
        let previous_to = to.saturating_sub(self.lookback);
        // Finish the historical read before borrowing current chunks: a source
        // may hold a non-reentrant publication lock during its callback.
        let previous = self.source.collect_range_dyn(previous_from, previous_to);
        self.source.for_each_chunk_at(from, to, &mut |at, current| {
            let prefix = self.lookback.saturating_sub(at).min(current.len());
            if prefix > 0 {
                each(at, &current[..prefix], None);
            }
            let current = &current[prefix..];
            if current.is_empty() {
                return;
            }
            let at = at + prefix;
            let offset = at - self.lookback - previous_from;
            each(at, current, Some(&previous[offset..offset + current.len()]));
        });
    }
}

impl<I, S, T> Clone for LazyLookbackVec<I, S, T>
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
            lookback: self.lookback,
            compute: Arc::clone(&self.compute),
        }
    }
}

impl<I, S, T> AnyVec for LazyLookbackVec<I, S, T>
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

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}

impl<I, S, T> TypedVec for LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    type I = I;
    type T = T;
}

impl<I, S, T> ReadableVec<I, T> for LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |_, current, previous| {
            self.compute.append(current, previous, buf);
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
                self.compute.append(current, previous, &mut output);
                f(at + offset, &output);
            }
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_chunk_at(from, to, &mut |_, values| {
            values.iter().cloned().for_each(&mut *f);
        });
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_chunk_at(from, to, &mut |_, values| {
            acc = Some(values.iter().cloned().fold(acc.take().unwrap(), &mut f));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        self.try_fold_lookback(from, to, init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let current = self.source.collect_one_at(index)?;
        let previous = index
            .checked_sub(self.lookback)
            .and_then(|index| self.source.collect_one_at(index));
        Some(self.compute.apply(current, previous))
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
            out.push(self.compute.apply(current, previous));
        }
    }
}

impl<I, S, T> Traversable for LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
