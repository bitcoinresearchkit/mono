use std::sync::Arc;

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Formattable, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex,
    VecValue, Version, short_type_name,
};

use super::SparseRead;

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
    compute: fn(S, Option<S>) -> T,
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
        source: ReadableBoxedVec<I, S>,
        lookback: usize,
        compute: fn(S, Option<S>) -> T,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            lookback,
            compute,
        }
    }
}

impl<I, S, T> LazyLookbackVec<I, S, T>
where
    I: VecIndex,
    S: VecValue,
    T: VecValue,
{
    fn for_each_lookback(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let to = to.min(self.len());
        if from >= to {
            return;
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
            each((self.compute)(current, previous));
        }
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
            compute: self.compute,
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
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_lookback(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_lookback(from, to, f);
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
        let current = self.source.collect_one_at(index)?;
        let previous = index
            .checked_sub(self.lookback)
            .and_then(|index| self.source.collect_one_at(index));
        Some((self.compute)(current, previous))
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
        let source = SparseRead::new(
            &*self.source,
            indices.iter().flat_map(|&index| {
                [Some(index), index.checked_sub(self.lookback)]
                    .into_iter()
                    .flatten()
            }),
        );

        out.reserve(indices.len());
        for &index in indices {
            let current = source.at(index);
            let previous = index
                .checked_sub(self.lookback)
                .map(|previous| source.at(previous));
            out.push((self.compute)(current, previous));
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
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, T, _>(self)
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Height, StoredU64, Version};
    use vecdb::{
        AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec, ReadableVec,
        WritableVec,
    };

    use super::LazyLookbackVec;

    #[test]
    fn sorted_reads_batch_current_and_lookback_values() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("brk-lazy-lookback-{}-{suffix}", std::process::id()));
        let db = Database::open(&path).unwrap();
        let mut source: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "source", Version::ONE).unwrap();

        for value in [2_u64, 5, 9, 14, 20] {
            source.push(StoredU64::from(value));
        }
        source.write().unwrap();

        let lookback = LazyLookbackVec::new(
            "lookback",
            Version::ONE,
            source.read_only_boxed_clone(),
            2,
            |current, previous| current - previous.unwrap_or_default(),
        );

        assert_eq!(
            lookback.read_sorted_at(&[0, 2, 2, 4, 5]),
            [2_u64, 7, 7, 11].map(StoredU64::from)
        );
        assert_eq!(lookback.read_sorted_at(&[4]), [11_u64].map(StoredU64::from));

        drop(lookback);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
