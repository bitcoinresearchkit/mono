use std::{marker::PhantomData, sync::Arc};

use brk_types::{Height, StoredU64};
use vecdb::{
    AnyVec, BinaryTransform, CachedBoxedVec, CheckedSub, PrintableIndex, ReadableBoxedVec,
    ReadableVec, TypedVec, VecIndex, VecValue, Version, short_type_name,
};

use crate::internal::CachedBlockCountReader;

pub struct LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    numerator: ReadableBoxedVec<Height, StoredU64>,
    denominator: CachedBlockCountReader,
    window_starts: CachedBoxedVec<Height, Height>,
    _output: PhantomData<(T, F)>,
}

impl<T, F> LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        numerator: ReadableBoxedVec<Height, StoredU64>,
        denominator: CachedBlockCountReader,
        window_starts: CachedBoxedVec<Height, Height>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            numerator,
            denominator,
            window_starts,
            _output: PhantomData,
        }
    }
}

impl<T, F> LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T>,
{
    #[inline(always)]
    fn previous_index(start: Height) -> Option<usize> {
        start.to_usize().checked_sub(1)
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(T)) {
        let window_starts = self.window_starts.snapshot();
        let to = to
            .min(self.numerator.len())
            .min(self.denominator.len())
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
            let numerators = self.numerator.collect_range_dyn(read_from, to);
            let mut offset = 0;
            self.denominator
                .for_each_rolling_sum(from, starts, |denominator| {
                    let index = from + offset;
                    let current = numerators[index - read_from];
                    let previous = Self::previous_index(starts[offset])
                        .map(|previous| numerators[previous - read_from])
                        .unwrap_or_default();
                    each(F::apply(
                        current.checked_sub(previous).unwrap_or_default(),
                        denominator,
                    ));
                    offset += 1;
                });
            return;
        }

        let current = self.numerator.collect_range_dyn(from, to);
        let previous = first_previous
            .zip(last_previous)
            .map(|(first, last)| (first, self.numerator.collect_range_dyn(first, last + 1)));
        let mut offset = 0;

        self.denominator
            .for_each_rolling_sum(from, starts, |denominator| {
                let previous = Self::previous_index(starts[offset])
                    .and_then(|index| {
                        previous
                            .as_ref()
                            .map(|(first, values)| values[index - first])
                    })
                    .unwrap_or_default();
                each(F::apply(
                    current[offset].checked_sub(previous).unwrap_or_default(),
                    denominator,
                ));
                offset += 1;
            });
    }
}

impl<T, F> Clone for LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            base_version: self.base_version,
            numerator: self.numerator.clone(),
            denominator: self.denominator.clone(),
            window_starts: self.window_starts.clone(),
            _output: PhantomData,
        }
    }
}

impl<T, F> AnyVec for LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
{
    fn version(&self) -> Version {
        self.base_version
            + self.numerator.version()
            + self.denominator.version()
            + self.window_starts.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.numerator
            .len()
            .min(self.denominator.len())
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

impl<T, F> TypedVec for LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
{
    type I = Height;
    type T = T;
}

impl<T, F> ReadableVec<Height, T> for LazyRollingRatioWithCachedBlockCount<T, F>
where
    T: VecValue,
    F: BinaryTransform<StoredU64, StoredU64, T> + Send + Sync,
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

        let start = self.window_starts.snapshot()[index];
        let previous = Self::previous_index(start);
        Some(F::apply(
            self.numerator
                .collect_one_at(index)?
                .checked_sub(
                    previous
                        .and_then(|index| self.numerator.collect_one_at(index))
                        .unwrap_or_default(),
                )
                .unwrap_or_default(),
            self.denominator.rolling_sum_at(start.to_usize(), index)?,
        ))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<T>) {
        out.reserve(indices.len());
        indices
            .iter()
            .filter_map(|&index| self.collect_one_at(index))
            .for_each(|value| out.push(value));
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{PartsPerMillion32, StoredU16};
    use vecdb::{
        AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec,
        WritableVec,
    };

    use super::*;
    use crate::internal::{LazyRatioWithCachedBlockCount, RatioU64};

    #[test]
    fn derives_cumulative_and_rolling_ratios_from_compact_counts() {
        let path = std::env::temp_dir().join(format!(
            "brk-cached-block-count-ratio-{}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut numerator: EagerVec<PcoVec<Height, StoredU64>> =
            EagerVec::forced_import(&db, "numerator", Version::ONE).unwrap();
        let mut denominator: EagerVec<PcoVec<Height, StoredU16>> =
            EagerVec::forced_import(&db, "denominator", Version::ONE).unwrap();
        let mut starts: EagerVec<PcoVec<Height, Height>> =
            EagerVec::forced_import(&db, "starts", Version::ONE).unwrap();

        for value in [10_u64, 30, 60, 100] {
            numerator.push(StoredU64::from(value));
        }
        for value in [20_u16, 30, 40, 50] {
            denominator.push(StoredU16::new(value));
        }
        for value in [0, 0, 1, 2] {
            starts.push(Height::new(value));
        }
        numerator.write().unwrap();
        denominator.write().unwrap();
        starts.write().unwrap();

        let denominator = CachedBlockCountReader::new(
            CachedVec::wrap(denominator).read_only_cached_boxed_clone(),
        );
        let cumulative =
            LazyRatioWithCachedBlockCount::<PartsPerMillion32, RatioU64<PartsPerMillion32>>::new(
                "cumulative",
                Version::ONE,
                numerator.read_only_boxed_clone(),
                denominator.clone(),
            );
        let starts = CachedVec::wrap(starts);
        let rolling = LazyRollingRatioWithCachedBlockCount::<
            PartsPerMillion32,
            RatioU64<PartsPerMillion32>,
        >::new(
            "rolling",
            Version::ONE,
            numerator.read_only_boxed_clone(),
            denominator,
            starts.read_only_cached_boxed_clone(),
        );

        assert_eq!(
            cumulative.collect_range_at(0, 4),
            [0.5, 0.6, 2.0 / 3.0, 5.0 / 7.0].map(PartsPerMillion32::from)
        );
        assert_eq!(
            rolling.collect_range_at(0, 4),
            [0.5, 0.6, 5.0 / 7.0, 7.0 / 9.0].map(PartsPerMillion32::from)
        );

        drop(rolling);
        drop(cumulative);
        drop(starts);
        drop(numerator);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
