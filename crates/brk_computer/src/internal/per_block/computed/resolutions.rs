use std::marker::PhantomData;

use brk_traversable::Traversable;
use brk_types::{
    Day1, Day3, Epoch, FromCoarserIndex, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30,
    Month1, Month3, Month6, Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    AggFold, CachedVec, LazyAggVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, StoredVec, TypedVec, VecIndex, VecValue,
};

use crate::{
    indexes,
    internal::{PerResolution, cache_wrap},
};

/// Aggregation strategy for epoch-based indices (Halving, Epoch).
///
/// Uses `FromCoarserIndex::max_from` to compute the target height for each
/// coarse index, rather than reading from the mapping. The mapping is only
/// used for its length.
pub struct CoarserIndex<I>(PhantomData<I>);

impl<I, O, S1I, S2T> AggFold<O, S1I, S2T, O> for CoarserIndex<I>
where
    I: VecIndex,
    O: VecValue,
    S1I: VecIndex + FromCoarserIndex<I>,
    S2T: VecValue,
{
    #[inline]
    fn try_fold<S: ReadableVec<S1I, O> + ?Sized, B, E, F: FnMut(B, O) -> Result<B, E>>(
        source: &S,
        mapping: &[S2T],
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        let mapping_len = mapping.len();
        let source_len = source.len();

        let indices: Vec<usize> = (from..to.min(mapping_len))
            .map(|i| S1I::max_from(I::from(i), source_len))
            .collect();

        let values = source.read_sorted_at(&indices);

        values.into_iter().try_fold(init, f)
    }

    #[inline]
    fn collect_one<S: ReadableVec<S1I, O> + ?Sized>(
        source: &S,
        _mapping: &[S2T],
        index: usize,
    ) -> Option<O> {
        let target = S1I::max_from(I::from(index), source.len());
        source.collect_one_at(target)
    }
}

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct Resolutions<T>(
    pub  PerResolution<
        LazyAggVec<Minute10, Option<T>, Height, Height, T>,
        LazyAggVec<Minute30, Option<T>, Height, Height, T>,
        LazyAggVec<Hour1, Option<T>, Height, Height, T>,
        LazyAggVec<Hour4, Option<T>, Height, Height, T>,
        LazyAggVec<Hour12, Option<T>, Height, Height, T>,
        LazyAggVec<Day1, Option<T>, Height, Height, T>,
        LazyAggVec<Day3, Option<T>, Height, Height, T>,
        LazyAggVec<Week1, Option<T>, Height, Height, T>,
        LazyAggVec<Month1, Option<T>, Height, Height, T>,
        LazyAggVec<Month3, Option<T>, Height, Height, T>,
        LazyAggVec<Month6, Option<T>, Height, Height, T>,
        LazyAggVec<Year1, Option<T>, Height, Height, T>,
        LazyAggVec<Year10, Option<T>, Height, Height, T>,
        LazyAggVec<Halving, T, Height, Height, T, CoarserIndex<Halving>>,
        LazyAggVec<Epoch, T, Height, Height, T, CoarserIndex<Epoch>>,
    >,
)
where
    T: VecValue + PartialOrd + JsonSchema;

impl<T> ReadOnlyClone for Resolutions<T>
where
    T: VecValue + PartialOrd + JsonSchema,
{
    type ReadOnly = Self;
    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl<T> Resolutions<T>
where
    T: VecValue + PartialOrd + JsonSchema + 'static,
{
    pub(crate) fn from_cached_height<V>(
        name: &str,
        height_source: &CachedVec<V>,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        V: StoredVec<I = Height, T = T>,
    {
        Self::from_boxed_height_source(
            name,
            height_source.read_only_boxed_clone(),
            version,
            indexes,
        )
    }

    pub(crate) fn forced_import<V>(
        name: &str,
        height_source: V,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        V: TypedVec<I = Height, T = T> + ReadableVec<Height, T> + Clone + 'static,
    {
        let cached = cache_wrap(height_source);
        let height_source = cached.read_only_boxed_clone();
        Self::from_boxed_height_source(name, height_source, version, indexes)
    }

    /// Build resolution views without materializing the height source in the
    /// budgeted cache.
    ///
    /// This is for sources that already read from compact in-memory state.
    pub(crate) fn forced_import_uncached<V>(
        name: &str,
        height_source: V,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Self
    where
        V: TypedVec<I = Height, T = T> + ReadableVec<Height, T> + Clone + 'static,
    {
        Self::from_boxed_height_source(
            name,
            height_source.read_only_boxed_clone(),
            version,
            indexes,
        )
    }

    fn from_boxed_height_source(
        name: &str,
        height_source: ReadableBoxedVec<Height, T>,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Self {
        macro_rules! res {
            ($field:expr) => {{
                let cached = $field.clone();
                let mapping_version = cached.version();
                LazyAggVec::new(
                    name,
                    version,
                    mapping_version,
                    height_source.clone(),
                    move || cached.cached(),
                )
            }};
        }

        Self(PerResolution {
            minute10: res!(indexes.minute10.first_height),
            minute30: res!(indexes.minute30.first_height),
            hour1: res!(indexes.hour1.first_height),
            hour4: res!(indexes.hour4.first_height),
            hour12: res!(indexes.hour12.first_height),
            day1: res!(indexes.day1.first_height),
            day3: res!(indexes.day3.first_height),
            week1: res!(indexes.week1.first_height),
            month1: res!(indexes.month1.first_height),
            month3: res!(indexes.month3.first_height),
            month6: res!(indexes.month6.first_height),
            year1: res!(indexes.year1.first_height),
            year10: res!(indexes.year10.first_height),
            halving: res!(indexes.halving.first_height),
            epoch: res!(indexes.epoch.first_height),
        })
    }
}
