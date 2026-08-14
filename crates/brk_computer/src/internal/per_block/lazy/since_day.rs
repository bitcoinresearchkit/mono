use std::sync::Arc;

use brk_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::{Day1, Height};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, CachedBoxedVec, Formattable, PrintableIndex, ReadableBoxedVec,
    ReadableVec, TypedVec, VecValue, Version, short_type_name,
};

/// Lazily derives values accumulated since a fixed day from one cumulative source.
pub struct LazySinceDayVec<S, T>
where
    S: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<Height, S>,
    days: CachedBoxedVec<Height, Day1>,
    start_day: Day1,
    compute: Arc<dyn Fn(S, S) -> T + Send + Sync>,
}

impl<S, T> LazySinceDayVec<S, T>
where
    S: VecValue + Default,
    T: VecValue + Default,
{
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, S>,
        days: CachedBoxedVec<Height, Day1>,
        start_day: Day1,
        compute: impl Fn(S, S) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            days,
            start_day,
            compute: Arc::new(compute),
        }
    }

    fn start_height(&self) -> usize {
        let days = self.days.snapshot();
        let mut left = 0;
        let mut right = self.len().min(days.len());
        while left < right {
            let middle = left + (right - left) / 2;
            if days[middle] < self.start_day {
                left = middle + 1;
            } else {
                right = middle;
            }
        }
        left
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let to = to.min(self.len());
        if from >= to {
            return;
        }

        let start = self.start_height();
        let active_from = from.max(start).min(to);
        for _ in from..active_from {
            each(T::default());
        }

        if active_from == to {
            return;
        }

        let before = start
            .checked_sub(1)
            .and_then(|index| self.source.collect_one_at(index))
            .unwrap_or_default();
        for current in self.source.collect_range_dyn(active_from, to) {
            each((self.compute)(current, before.clone()));
        }
    }
}

impl<S, T> Clone for LazySinceDayVec<S, T>
where
    S: VecValue,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            days: self.days.clone(),
            start_day: self.start_day,
            compute: Arc::clone(&self.compute),
        }
    }
}

impl<S, T> AnyVec for LazySinceDayVec<S, T>
where
    S: VecValue,
    T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version() + self.days.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
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

impl<S, T> TypedVec for LazySinceDayVec<S, T>
where
    S: VecValue,
    T: VecValue,
{
    type I = Height;
    type T = T;
}

impl<S, T> ReadableVec<Height, T> for LazySinceDayVec<S, T>
where
    S: VecValue + Default,
    T: VecValue + Default,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_value(from, to, f);
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
        if index >= self.len() {
            return None;
        }

        let start = self.start_height();
        if index < start {
            return Some(T::default());
        }

        let current = self.source.collect_one_at(index)?;
        let before = start
            .checked_sub(1)
            .and_then(|index| self.source.collect_one_at(index))
            .unwrap_or_default();
        Some((self.compute)(current, before))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        out.reserve(indices.len());
        indices
            .iter()
            .filter_map(|&index| self.collect_one_at(index))
            .for_each(|value| out.push(value));
    }
}

impl<S, T> Traversable for LazySinceDayVec<S, T>
where
    S: VecValue + Default,
    T: VecValue + Default + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<Height, T, _>(self)
    }
}
