use std::{iter::once, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, READ_CHUNK_SIZE, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, TypedVec, VecIndex, VecValue, Version, short_type_name,
};

use vecdb::SparseRead;

struct WindowInputs<'a, I, S> {
    at: usize,
    starts: &'a [I],
    previous: &'a [S],
    previous_from: usize,
    inclusive: bool,
}

trait WindowTransform<I, S, T>: Send + Sync {
    fn apply(&self, current: S, previous: S, count: usize) -> T;
    fn append(&self, current: &[S], inputs: &WindowInputs<I, S>, out: &mut Vec<T>);
}

impl<I, S, T, F> WindowTransform<I, S, T> for F
where
    I: VecIndex,
    S: VecValue + Default,
    F: Fn(S, S, usize) -> T + Send + Sync,
{
    fn apply(&self, current: S, previous: S, count: usize) -> T {
        self(current, previous, count)
    }
    fn append(&self, current: &[S], inputs: &WindowInputs<I, S>, out: &mut Vec<T>) {
        out.extend(
            (inputs.at..inputs.at + current.len())
                .zip(current.iter().cloned())
                .zip(inputs.starts)
                .map(|((at, current), start)| {
                    let start = start.to_usize();
                    let ago = if inputs.inclusive {
                        start.checked_sub(1)
                    } else {
                        Some(start)
                    };
                    let previous = ago
                        .map(|i| inputs.previous[i - inputs.previous_from].clone())
                        .unwrap_or_default();
                    self(
                        current,
                        previous,
                        at - start + usize::from(inputs.inclusive),
                    )
                }),
        );
    }
}

/// Lazily combines the current and window-start values of one metric source.
///
/// `window_starts` is index metadata, not a second metric source. Reads are
/// split into current and lookback ranges so long windows do not force reading
/// the entire gap between them.
pub struct LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    window_starts: ReadableBoxedVec<I, I>,
    inclusive: bool,
    compute: Arc<dyn WindowTransform<I, S, T>>,
}

impl<I, S, T> LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue + Default,
    T: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<I, S> + ?Sized),
        window_starts: &(impl ReadableCloneableVec<I, I> + ?Sized),
        inclusive: bool,
        compute: impl Fn(S, S, usize) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source: source.read_only_boxed_clone(),
            window_starts: window_starts.read_only_boxed_clone(),
            inclusive,
            compute: Arc::new(compute),
        }
    }

    #[inline]
    fn ago_index(&self, start: usize) -> Option<usize> {
        if self.inclusive {
            start.checked_sub(1)
        } else {
            Some(start)
        }
    }

    #[inline]
    fn count(&self, current: usize, start: usize) -> usize {
        if self.inclusive {
            current - start + 1
        } else {
            current - start
        }
    }

    fn try_fold_window<B, E>(
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

        let window_starts = self.window_starts.collect_range_dyn(from, to);
        let starts = window_starts.as_slice();
        let current = self.source.collect_range_dyn(from, to);

        let first_ago = starts
            .iter()
            .find_map(|start| self.ago_index(start.to_usize()));
        let last_ago = starts
            .iter()
            .rev()
            .find_map(|start| self.ago_index(start.to_usize()));
        let ago = first_ago
            .zip(last_ago)
            .map(|(first, last)| self.source.collect_range_dyn(first, last + 1));

        for (offset, (current, start)) in current.into_iter().zip(starts).enumerate() {
            let start = start.to_usize();
            let previous = self
                .ago_index(start)
                .and_then(|index| {
                    ago.as_ref()
                        .zip(first_ago)
                        .map(|(values, first)| values[index - first].clone())
                })
                .unwrap_or_default();
            accumulator = fold(
                accumulator,
                self.compute
                    .apply(current, previous, self.count(from + offset, start)),
            )?;
        }
        Ok(accumulator)
    }

    fn for_each_input(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(&[S], WindowInputs<I, S>),
    ) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }
        let window_starts = self.window_starts.collect_range_dyn(from, to);
        let starts = window_starts.as_slice();
        let first = starts
            .iter()
            .find_map(|start| self.ago_index(start.to_usize()));
        let last = starts
            .iter()
            .rev()
            .find_map(|start| self.ago_index(start.to_usize()));
        let (previous_from, previous_to) = first
            .zip(last)
            .map(|(first, last)| (first, last + 1))
            .unwrap_or((0, 0));
        // Collect prior values before borrowing the current source chunks.
        let previous = self.source.collect_range_dyn(previous_from, previous_to);
        self.source.for_each_chunk_at(from, to, &mut |at, current| {
            each(
                current,
                WindowInputs {
                    at,
                    starts: &window_starts[at - from..at - from + current.len()],
                    previous: &previous,
                    previous_from,
                    inclusive: self.inclusive,
                },
            );
        });
    }
}

impl<I, S, T> Clone for LazyWindowVec<I, S, T>
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
            window_starts: self.window_starts.clone(),
            inclusive: self.inclusive,
            compute: Arc::clone(&self.compute),
        }
    }
}

impl<I, S, T> AnyVec for LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version() + self.window_starts.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        self.source.len().min(self.window_starts.len())
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

impl<I, S, T> TypedVec for LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    type I = I;
    type T = T;
}

impl<I, S, T> ReadableVec<I, T> for LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue + Default,
    T: VecValue,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |current, inputs| {
            self.compute.append(current, &inputs, buf);
        });
    }

    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[T])) {
        let chunk_size = self.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        let mut output = Vec::new();
        self.for_each_input(from, to, |current, inputs| {
            for (chunk, current) in current.chunks(chunk_size).enumerate() {
                let offset = chunk * chunk_size;
                let inputs = WindowInputs {
                    at: inputs.at + offset,
                    starts: &inputs.starts[offset..offset + current.len()],
                    ..inputs
                };
                output.clear();
                self.compute.append(current, &inputs, &mut output);
                f(inputs.at, &output);
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
        self.try_fold_window(from, to, init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let start = self.window_starts.collect_one_at(index)?.to_usize();
        let current = self.source.collect_one_at(index)?;
        let previous = self
            .ago_index(start)
            .and_then(|index| self.source.collect_one_at(index))
            .unwrap_or_default();
        Some(
            self.compute
                .apply(current, previous, self.count(index, start)),
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

        let len = self.len();
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        if indices.is_empty() {
            return;
        }

        let window_starts = self.window_starts.read_sorted_at(indices);
        let mut starts = window_starts.iter();
        let source = SparseRead::new(&*self.source, indices, |_| {
            self.ago_index(starts.next().unwrap().to_usize())
        });

        out.reserve(indices.len());
        for (output, &index) in indices.iter().enumerate() {
            let start = window_starts[output].to_usize();
            let previous = source.previous(output).unwrap_or_default();
            out.push(self.compute.apply(
                source.current(output),
                previous,
                self.count(index, start),
            ));
        }
    }
}

impl<I, S, T> Traversable for LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue + Default,
    T: VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
