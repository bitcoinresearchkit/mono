use std::sync::Arc;

use brk_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, CachedBoxedVec, Formattable, ReadableBoxedVec, ReadableVec, TypedVec,
    VecIndex, VecValue, Version, short_type_name,
};

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
    window_starts: CachedBoxedVec<I, I>,
    inclusive: bool,
    compute: Arc<dyn Fn(S, S, usize) -> T + Send + Sync>,
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
        source: ReadableBoxedVec<I, S>,
        window_starts: CachedBoxedVec<I, I>,
        inclusive: bool,
        compute: impl Fn(S, S, usize) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            window_starts,
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

    fn for_each_window(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let window_starts = self.window_starts.snapshot();
        let to = to.min(self.len()).min(window_starts.len());
        if from >= to {
            return;
        }

        let starts = &window_starts[from..to];
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
            each((self.compute)(
                current,
                previous,
                self.count(from + offset, start),
            ));
        }
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
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_window(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_window(from, to, f);
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().fold(init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().try_fold(init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let window_starts = self.window_starts.snapshot();
        let start = window_starts.get(index)?.to_usize();
        let current = self.source.collect_one_at(index)?;
        let previous = self
            .ago_index(start)
            .and_then(|index| self.source.collect_one_at(index))
            .unwrap_or_default();
        Some((self.compute)(current, previous, self.count(index, start)))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        out.reserve(indices.len());
        indices
            .iter()
            .filter_map(|&index| self.collect_one_at(index))
            .for_each(|value| out.push(value));
    }
}

impl<I, S, T> Traversable for LazyWindowVec<I, S, T>
where
    I: VecIndex,
    S: VecValue + Default,
    T: VecValue + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}
