use std::{marker::PhantomData, sync::Arc};

use brk_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::Height;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, BinaryTransform, CachedBoxedVec, CheckedSub, Formattable,
    PrintableIndex, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex, VecValue, Version,
    short_type_name,
};

/// Rolling transform derived from one cumulative source and one cached
/// cumulative operand.
///
/// Only `source` may read from disk. The cached operand and window starts are
/// pinned in-memory snapshots shared with their authoritative sources.
pub struct LazyRollingRatioVec<S, C, T, F>
where
    S: VecValue,
    C: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<Height, S>,
    cached: CachedBoxedVec<Height, C>,
    cached_transform: fn(Height, C) -> C,
    window_starts: CachedBoxedVec<Height, Height>,
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
    fn identity_cached(_: Height, value: C) -> C {
        value
    }

    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, S>,
        cached: CachedBoxedVec<Height, C>,
        window_starts: CachedBoxedVec<Height, Height>,
    ) -> Self {
        Self::with_cached_transform(
            name,
            version,
            source,
            cached,
            window_starts,
            Self::identity_cached,
        )
    }

    pub fn with_cached_transform(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, S>,
        cached: CachedBoxedVec<Height, C>,
        window_starts: CachedBoxedVec<Height, Height>,
        cached_transform: fn(Height, C) -> C,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            cached,
            cached_transform,
            window_starts,
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
        cached_current: C,
        cached_previous: C,
    ) -> T {
        F::apply(
            source_current
                .checked_sub(source_previous)
                .unwrap_or_default(),
            (self.cached_transform)(Height::from(index), cached_current)
                .checked_sub(
                    previous
                        .map(|index| (self.cached_transform)(Height::from(index), cached_previous))
                        .unwrap_or_default(),
                )
                .unwrap_or_default(),
        )
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let cached = self.cached.snapshot();
        let window_starts = self.window_starts.snapshot();
        let to = to
            .min(self.source.len())
            .min(cached.len())
            .min(window_starts.len());
        if from >= to {
            return;
        }

        let starts = &window_starts[from..to];
        let first_previous = starts.iter().find_map(|start| Self::previous_index(*start));
        let last_previous = starts
            .iter()
            .rev()
            .find_map(|start| Self::previous_index(*start));

        if let Some((first_previous, last_previous)) = first_previous.zip(last_previous)
            && last_previous + 1 >= from
        {
            let read_from = first_previous.min(from);
            let source = self.source.collect_range_dyn(read_from, to);
            for (offset, start) in starts.iter().enumerate() {
                let index = from + offset;
                let source_current = source[index - read_from].clone();
                let previous = Self::previous_index(*start);
                let source_previous = previous
                    .map(|previous| source[previous - read_from].clone())
                    .unwrap_or_default();
                let cached_current = cached[index].clone();
                let cached_previous = previous
                    .map(|previous| cached[previous].clone())
                    .unwrap_or_default();
                each(self.compute(
                    index,
                    previous,
                    source_current,
                    source_previous,
                    cached_current,
                    cached_previous,
                ));
            }
            return;
        }

        let current = self.source.collect_range_dyn(from, to);
        let previous = first_previous
            .zip(last_previous)
            .map(|(first, last)| (first, self.source.collect_range_dyn(first, last + 1)));

        for (offset, (source_current, start)) in current.into_iter().zip(starts).enumerate() {
            let index = from + offset;
            let previous_index = Self::previous_index(*start);
            let source_previous = previous_index
                .and_then(|previous_index| {
                    previous
                        .as_ref()
                        .map(|(first, values)| values[previous_index - first].clone())
                })
                .unwrap_or_default();
            let cached_current = cached[index].clone();
            let cached_previous = previous_index
                .map(|previous| cached[previous].clone())
                .unwrap_or_default();
            each(self.compute(
                index,
                previous_index,
                source_current,
                source_previous,
                cached_current,
                cached_previous,
            ));
        }
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
            cached: self.cached.clone(),
            cached_transform: self.cached_transform,
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
            + self.cached.version()
            + self.window_starts.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source
            .len()
            .min(self.cached.len())
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
        buf.reserve(to.saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, G: FnMut(B, T) -> B>(&self, from: usize, to: usize, init: B, fold: G) -> B {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().fold(init, fold)
    }

    fn try_fold_range_at<B, E, G: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: G,
    ) -> Result<B, E> {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().try_fold(init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }

        let cached = self.cached.snapshot();
        let window_starts = self.window_starts.snapshot();
        let previous = Self::previous_index(window_starts[index]);
        Some(
            self.compute(
                index,
                previous,
                self.source.collect_one_at(index)?,
                previous
                    .and_then(|index| self.source.collect_one_at(index))
                    .unwrap_or_default(),
                cached[index].clone(),
                previous
                    .map(|index| cached[index].clone())
                    .unwrap_or_default(),
            ),
        )
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        out.reserve(indices.len());
        indices
            .iter()
            .filter_map(|&index| self.collect_one_at(index))
            .for_each(|value| out.push(value));
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
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<Height, T, _>(self)
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{PartsPerMillion32, Sats};
    use vecdb::{
        AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec,
        ReadableVec, WritableVec,
    };

    use super::*;
    use crate::internal::RatioSats;

    fn double(_: Height, value: Sats) -> Sats {
        value + value
    }

    #[test]
    fn derives_rolling_ratios_from_one_source_and_cached_denominator() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-lazy-rolling-ratio-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut source: EagerVec<PcoVec<Height, Sats>> =
            EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
        let mut denominator: EagerVec<PcoVec<Height, Sats>> =
            EagerVec::forced_import(&db, "denominator", Version::ONE).unwrap();
        let mut starts: EagerVec<PcoVec<Height, Height>> =
            EagerVec::forced_import(&db, "starts", Version::ONE).unwrap();

        for value in [10, 30, 60, 100] {
            source.push(Sats::new(value));
        }
        for value in [20, 50, 90, 140] {
            denominator.push(Sats::new(value));
        }
        for value in [0, 0, 1, 2] {
            starts.push(Height::new(value));
        }
        source.write().unwrap();
        denominator.write().unwrap();
        starts.write().unwrap();

        let denominator = CachedVec::wrap(denominator);
        let starts = CachedVec::wrap(starts);
        let ratio = LazyRollingRatioVec::<
            Sats,
            Sats,
            PartsPerMillion32,
            RatioSats<PartsPerMillion32>,
        >::new(
            "ratio",
            Version::ONE,
            source.read_only_boxed_clone(),
            denominator.read_only_cached_boxed_clone(),
            starts.read_only_cached_boxed_clone(),
        );

        assert_eq!(
            ratio.collect_range(Height::ZERO, Height::new(4)),
            vec![
                PartsPerMillion32::from(0.5),
                PartsPerMillion32::from(0.6),
                PartsPerMillion32::from(50.0 / 70.0),
                PartsPerMillion32::from(70.0 / 90.0),
            ],
        );
        assert_eq!(
            ratio.collect_range(Height::new(3), Height::new(4)),
            [PartsPerMillion32::from(70.0 / 90.0)],
        );

        let transformed = LazyRollingRatioVec::<
            Sats,
            Sats,
            PartsPerMillion32,
            RatioSats<PartsPerMillion32>,
        >::with_cached_transform(
            "transformed",
            Version::ONE,
            source.read_only_boxed_clone(),
            denominator.read_only_cached_boxed_clone(),
            starts.read_only_cached_boxed_clone(),
            double,
        );
        assert_eq!(
            transformed.collect_range(Height::ZERO, Height::new(4)),
            vec![
                PartsPerMillion32::from(0.25),
                PartsPerMillion32::from(0.3),
                PartsPerMillion32::from(25.0 / 70.0),
                PartsPerMillion32::from(35.0 / 90.0),
            ],
        );

        drop(transformed);
        drop(ratio);
        drop(starts);
        drop(denominator);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
