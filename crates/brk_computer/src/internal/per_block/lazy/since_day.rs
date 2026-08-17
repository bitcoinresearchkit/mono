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

        let start = self.start_height();
        let inactive_end = indices.partition_point(|&index| index < start);

        out.reserve(indices.len());
        for _ in &indices[..inactive_end] {
            out.push(T::default());
        }
        if inactive_end == indices.len() {
            return;
        }

        let before = start
            .checked_sub(1)
            .and_then(|index| self.source.collect_one_at(index))
            .unwrap_or_default();
        for current in self.source.read_sorted_at(&indices[inactive_end..]) {
            out.push((self.compute)(current, before.clone()));
        }
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

#[cfg(test)]
mod tests {
    use brk_types::{Day1, Height, StoredU64, Version};
    use vecdb::{
        AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec,
        ReadableVec, WritableVec,
    };

    use super::LazySinceDayVec;

    #[test]
    fn sorted_reads_reuse_the_fixed_start_and_handle_boundaries() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-lazy-since-day-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut source: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
        let mut days: EagerVec<PcoVec<Height, Day1>> =
            EagerVec::forced_import(&db, "days", Version::ONE).unwrap();

        for value in [10_u64, 30, 60, 100, 150] {
            source.push(StoredU64::from(value));
        }
        for day in [0, 0, 1, 1, 2] {
            days.push(Day1::from(day));
        }
        source.write().unwrap();
        days.write().unwrap();

        let days = CachedVec::wrap(days);
        let since_day = LazySinceDayVec::new(
            "since_day",
            Version::ONE,
            source.read_only_boxed_clone(),
            days.read_only_cached_boxed_clone(),
            Day1::from(1),
            |current, before| current - before,
        );

        assert_eq!(
            since_day.read_sorted_at(&[0, 1, 2, 2, 4, 5]),
            [0_u64, 0, 30, 30, 120].map(StoredU64::from),
        );
        assert_eq!(
            since_day.read_sorted_at(&[2]),
            [30_u64].map(StoredU64::from),
        );
        assert_eq!(since_day.read_sorted_at(&[5]), []);

        drop(since_day);
        drop(days);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
