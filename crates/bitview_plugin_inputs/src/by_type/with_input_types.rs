//! Generic `all` + per-input-type container (11 spendable types — no
//! op_return since op_return outputs are non-spendable).

use bitview_cohort::{ByAddrType, Filter, SpendableType, SpendableTypeId};
use bitview_compute::{
    CachedBlockCountReader, CachedWindowStartVec, LazyColumnCountPerBlockCumulativeRolling,
    LazyColumnPerBlockCumulativeRolling, LazyPerBlockCumulativeRolling,
    LazyPercentCumulativeRolling, RatioU64, Windows,
};
use bitview_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, StoredU16, StoredU64, Version};
use vecdb::{
    CachedBoxedVec, CachedReadableVec, CachedVec, LazyVec, PcoVec, ReadOnlyColumnarVec,
    ReadableCloneableVec, ReadableVec, TypedVec,
};

/// `all` aggregate plus per-input-type breakdown across the 11 spendable
/// output types. The "type" of an input is the previous output it spends.
#[derive(Clone, Traversable)]
pub struct WithInputTypes<V> {
    pub all: LazyPerBlockCumulativeRolling<StoredU64>,
    #[traversable(skip)]
    cached_all: CachedBoxedVec<Height, StoredU64>,
    #[traversable(flatten)]
    pub by_type: SpendableType<V>,
}

impl<V> WithInputTypes<V> {
    fn new<S>(
        all_name: &str,
        version: Version,
        (all_source, all_transform): (S, fn(Height, StoredU64) -> StoredU64),
        by_type: SpendableType<V>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self
    where
        S: TypedVec<I = Height, T = StoredU64>
            + ReadableVec<Height, StoredU64>
            + CachedReadableVec<Height, StoredU64>
            + Clone
            + 'static,
    {
        let source = LazyVec::init(
            &format!("{all_name}_cumulative_source"),
            version,
            all_source.read_only_boxed_clone(),
            all_transform,
        );
        let all = LazyPerBlockCumulativeRolling::from_cumulative_source(
            all_name,
            version,
            source,
            cached_starts,
            mappings,
        );
        let cached_all = CachedVec::wrap(all.cumulative.height.clone()).cached_boxed_clone();
        Self {
            all,
            cached_all,
            by_type,
        }
    }

    pub fn lazy_share(
        &self,
        name: &str,
        version: Version,
        numerator: &(impl ReadableCloneableVec<Height, StoredU64> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> LazyPercentCumulativeRolling<PartsPerMillion32> {
        LazyPercentCumulativeRolling::from_cumulative_ratio::<
            StoredU64,
            StoredU64,
            RatioU64<PartsPerMillion32>,
        >(
            name,
            version,
            numerator,
            self.cached_all.clone(),
            cached_starts,
            mappings,
        )
    }
}

impl WithInputTypes<LazyColumnCountPerBlockCumulativeRolling> {
    pub fn from_columnar_count_source<S>(
        all_name: &str,
        per_type_name: impl Fn(&str) -> String,
        version: Version,
        all: (S, fn(Height, StoredU64) -> StoredU64),
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredU16>, SpendableTypeId>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self
    where
        S: TypedVec<I = Height, T = StoredU64>
            + ReadableVec<Height, StoredU64>
            + CachedReadableVec<Height, StoredU64>
            + Clone
            + 'static,
    {
        let by_type = SpendableType::new(|filter, name| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            LazyColumnCountPerBlockCumulativeRolling::new(
                &per_type_name(name),
                version,
                source,
                SpendableTypeId::from_output_type(output_type)
                    .expect("spendable output type column"),
                mappings,
                cached_starts,
            )
        });
        Self::new(all_name, version, all, by_type, mappings, cached_starts)
    }

    pub fn lazy_shares(
        &self,
        version: Version,
        name: impl Fn(&str) -> String,
        cached_starts: &Windows<&CachedWindowStartVec>,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>> {
        SpendableType::new(|filter, type_name| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            let numerator = self.by_type.get(output_type).cached_cumulative();
            self.lazy_share(
                &name(type_name),
                version,
                &numerator,
                cached_starts,
                mappings,
            )
        })
    }

    pub fn cached_addr_type_counts(&self) -> ByAddrType<CachedBlockCountReader> {
        ByAddrType::new(|filter| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            self.by_type.get(output_type).cached_cumulative()
        })
    }

    pub fn invalidate(&self) {
        self.by_type.iter().for_each(|count| count.invalidate());
    }
}

impl WithInputTypes<LazyColumnPerBlockCumulativeRolling<StoredU64, SpendableTypeId>> {
    pub fn from_columnar_source<S>(
        all_name: &str,
        per_type_name: impl Fn(&str) -> String,
        version: Version,
        all: (S, fn(Height, StoredU64) -> StoredU64),
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, SpendableTypeId>,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self
    where
        S: TypedVec<I = Height, T = StoredU64>
            + ReadableVec<Height, StoredU64>
            + CachedReadableVec<Height, StoredU64>
            + Clone
            + 'static,
    {
        let by_type = SpendableType::new(|filter, name| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            LazyColumnPerBlockCumulativeRolling::new(
                &per_type_name(name),
                version,
                source,
                SpendableTypeId::from_output_type(output_type)
                    .expect("spendable output type column"),
                mappings,
                cached_starts,
            )
        });
        Self::new(all_name, version, all, by_type, mappings, cached_starts)
    }

    pub fn lazy_shares(
        &self,
        version: Version,
        name: impl Fn(&str) -> String,
        cached_starts: &Windows<&CachedWindowStartVec>,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> SpendableType<LazyPercentCumulativeRolling<PartsPerMillion32>> {
        SpendableType::new(|filter, type_name| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            self.lazy_share(
                &name(type_name),
                version,
                &self.by_type.get(output_type).cumulative.height,
                cached_starts,
                mappings,
            )
        })
    }
}
