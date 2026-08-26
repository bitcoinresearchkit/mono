use std::{convert::Infallible, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, CachedBoxedVec, Formattable, ReadableBoxedVec, ReadableVec, TypedVec,
    VecIndex, VecValue, Version, short_type_name,
};

/// Lazily transforms one metric source with aligned index metadata.
pub struct LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    metadata: CachedBoxedVec<I, M>,
    compute: Arc<dyn Fn(I, S, M) -> T + Send + Sync>,
}

impl<I, S, M, T> LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S>,
        metadata: CachedBoxedVec<I, M>,
        compute: impl Fn(I, S, M) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            metadata,
            compute: Arc::new(compute),
        }
    }

    fn try_fold_values<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: impl FnMut(B, T) -> Result<B, E>,
    ) -> Result<B, E> {
        let metadata = self.metadata.snapshot();
        let to = to.min(self.source.len()).min(metadata.len());
        if from >= to {
            return Ok(init);
        }

        let source = self.source.collect_range_dyn(from, to);
        source
            .into_iter()
            .zip(metadata[from..to].iter().cloned())
            .enumerate()
            .try_fold(init, |accumulator, (offset, (source, metadata))| {
                fold(
                    accumulator,
                    (self.compute)(I::from(from + offset), source, metadata),
                )
            })
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        self.try_fold_values(from, to, (), |(), value| {
            each(value);
            Ok::<_, Infallible>(())
        })
        .unwrap();
    }
}

impl<I, S, M, T> Clone for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            metadata: self.metadata.clone(),
            compute: Arc::clone(&self.compute),
        }
    }
}

impl<I, S, M, T> AnyVec for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    fn version(&self) -> Version {
        self.base_version + self.source.version() + self.metadata.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn len(&self) -> usize {
        self.source.len().min(self.metadata.len())
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

impl<I, S, M, T> TypedVec for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    type I = I;
    type T = T;
}

impl<I, S, M, T> ReadableVec<I, T> for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
    T: VecValue,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.for_each_value(from, to, f);
    }

    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        self.try_fold_values(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(fold(accumulator, value))
        })
        .unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.try_fold_values(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<T> {
        let source = self.source.collect_one_at(index)?;
        let metadata = self.metadata.snapshot().get(index)?.clone();
        Some((self.compute)(I::from(index), source, metadata))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        let metadata = self.metadata.snapshot();
        let len = self.source.len().min(metadata.len());
        let indices = &indices[..indices.partition_point(|&index| index < len)];
        let source = self.source.read_sorted_at(indices);

        out.reserve(source.len());
        for (&index, value) in indices.iter().zip(source) {
            out.push((self.compute)(
                I::from(index),
                value,
                metadata[index].clone(),
            ));
        }
    }
}

impl<I, S, M, T> Traversable for LazyIndexedVec<I, S, M, T>
where
    I: VecIndex,
    S: VecValue,
    M: VecValue,
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
        AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec,
        ReadableVec, VecIndex, WritableVec,
    };

    use super::LazyIndexedVec;

    #[test]
    fn folds_aligned_values_and_stops_on_error() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("brk-lazy-indexed-{}-{suffix}", std::process::id()));
        let db = Database::open(&path).unwrap();
        let mut source: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
        let mut metadata: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "metadata", Version::ONE).unwrap();

        for value in [10_u64, 20, 30, 40, 50] {
            source.push(StoredU64::from(value));
        }
        for value in [1_u64, 2, 3, 4] {
            metadata.push(StoredU64::from(value));
        }
        source.write().unwrap();
        metadata.write().unwrap();

        let metadata = CachedVec::wrap(metadata);
        let indexed = LazyIndexedVec::new(
            "indexed",
            Version::ONE,
            source.read_only_boxed_clone(),
            metadata.read_only_cached_boxed_clone(),
            |height, source, metadata| {
                StoredU64::from(height.to_usize() as u64 + *source + *metadata)
            },
        );

        assert_eq!(
            indexed.collect_range_at(1, 10),
            [23_u64, 35, 47].map(StoredU64::from)
        );
        assert_eq!(
            indexed.fold_range_at(1, 10, 0_u64, |sum, value| sum + *value),
            105
        );

        let mut visited = Vec::new();
        let result = indexed.try_fold_range_at(0, 10, 0_u64, |sum, value| {
            visited.push(*value);
            if *value == 35 {
                Err(35)
            } else {
                Ok(sum + *value)
            }
        });
        assert_eq!(result, Err(35));
        assert_eq!(visited, [11, 23, 35]);

        drop(indexed);
        drop(metadata);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
