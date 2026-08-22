//! Generic `all` + per-`OutputType` container (12 output types, including
//! op_return).

use bitview_cohort::{ByAddrType, ByType, Filter, OutputTypeId};
use bitview_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, StoredU16, StoredU64, Version};
use vecdb::{
    CachedBoxedVec, CachedReadableVec, LazyVec, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec,
    ReadableVec, TypedVec,
};

use bitview_compute::{
    CachedBlockCountReader, CachedWindowStartVec, LazyColumnCountPerBlockCumulativeRolling,
    LazyColumnPerBlockCumulativeRolling, LazyPerBlockCumulativeRolling,
    LazyPercentCumulativeRolling, RatioU64, Windows,
};

/// `all` aggregate plus per-`OutputType` breakdown across all 12 output
/// types (spendable + op_return).
#[derive(Clone, Traversable)]
pub struct WithOutputTypes<V> {
    /// Across all output types, including OP_RETURN outputs.
    pub all: LazyPerBlockCumulativeRolling<StoredU64>,
    #[traversable(skip)]
    cached_all: CachedBoxedVec<Height, StoredU64>,
    #[traversable(flatten)]
    pub by_type: ByType<V>,
}

impl<V> WithOutputTypes<V> {
    pub fn new<S>(
        all_name: &str,
        version: Version,
        (all_source, all_transform): (S, fn(Height, StoredU64) -> StoredU64),
        by_type: ByType<V>,
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
        let cached_all = all_source.cached_boxed_clone();
        let source = LazyVec::init(
            &format!("{all_name}_cumulative_source"),
            version,
            all_source.read_only_boxed_clone(),
            all_transform,
        );
        Self {
            all: LazyPerBlockCumulativeRolling::from_cumulative_source(
                all_name,
                version,
                source,
                cached_starts,
                mappings,
            ),
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

impl WithOutputTypes<LazyColumnCountPerBlockCumulativeRolling> {
    pub fn from_columnar_count_source<S>(
        all_name: &str,
        per_type_name: impl Fn(&str) -> String,
        version: Version,
        all: (S, fn(Height, StoredU64) -> StoredU64),
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredU16>, OutputTypeId>,
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
        let by_type = ByType::new(|filter, name| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            LazyColumnCountPerBlockCumulativeRolling::new(
                &per_type_name(name),
                version,
                source,
                OutputTypeId::from_output_type(output_type),
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
    ) -> ByType<LazyPercentCumulativeRolling<PartsPerMillion32>> {
        ByType::new(|filter, type_name| {
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

impl WithOutputTypes<LazyColumnPerBlockCumulativeRolling<StoredU64, OutputTypeId>> {
    pub fn from_columnar_source<S>(
        all_name: &str,
        per_type_name: impl Fn(&str) -> String,
        version: Version,
        all: (S, fn(Height, StoredU64) -> StoredU64),
        source: &ReadOnlyColumnarVec<PcoVec<Height, StoredU64>, OutputTypeId>,
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
        let by_type = ByType::new(|filter, name| {
            let Filter::Type(output_type) = filter else {
                unreachable!()
            };
            LazyColumnPerBlockCumulativeRolling::new(
                &per_type_name(name),
                version,
                source,
                OutputTypeId::from_output_type(output_type),
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
    ) -> ByType<LazyPercentCumulativeRolling<PartsPerMillion32>> {
        ByType::new(|filter, type_name| {
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
