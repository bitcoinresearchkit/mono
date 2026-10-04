use bitview_cohort::{ByAddrType, ByType, SpendableType};
use bitview_collections::Windows;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{LazyVec, ReadableCloneableVec};

use crate::{
    CountTotal, IndexSources, LazyFixedRatioCumulativeRolling, LazyPerBlockCumulativeRolling,
};

/// A shared total plus a typed count breakdown. The group determines membership;
/// the total component determines the denominator, independently of the per-type sources.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct TypeCounts<S> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    total: CountTotal,
    #[traversable(flatten)]
    pub by_type: S,
}

pub type SpendableTypeCounts<V> = TypeCounts<SpendableType<V>>;
pub type OutputTypeCounts<V> = TypeCounts<ByType<V>>;

// Both typed domains use the same vector assembly, without erasing either
// group's concrete fields or changing its iteration order.
macro_rules! impl_type_counts {
    ($group:ident) => {
        impl TypeCounts<$group<LazyPerBlockCumulativeRolling<Count>>> {
            pub fn addr_type_counts(&self) -> ByAddrType<LazyVec<Height, Count, Height, Count>> {
                ByAddrType::from_fn(|id| {
                    self.by_type.get(id.output_type()).cumulative.height.clone()
                })
            }

            pub fn from_cumulative_sources(
                total: CountTotal,
                per_type_name: impl Fn(&str) -> String,
                version: Version,
                sources: &$group<impl ReadableCloneableVec<Height, Count>>,
                indexes: &IndexSources,
                windows: &Windows<&impl ReadableCloneableVec<Height, Height>>,
            ) -> Self {
                let by_type = sources.map_with_id(|id, source| {
                    LazyPerBlockCumulativeRolling::from_cumulative_source(
                        &per_type_name(id.name()),
                        version,
                        source,
                        windows,
                        indexes,
                    )
                });
                Self { total, by_type }
            }

            pub fn lazy_shares(
                &self,
                version: Version,
                name: impl Fn(&str) -> String,
                windows: &Windows<&impl ReadableCloneableVec<Height, Height>>,
                indexes: &IndexSources,
            ) -> $group<LazyFixedRatioCumulativeRolling<PartsPerMillion32>> {
                self.by_type.map_with_id(|id, source| {
                    self.total.lazy_share(
                        &name(id.name()),
                        version,
                        &source.cumulative.height,
                        windows,
                        indexes,
                    )
                })
            }
        }
    };
}

impl_type_counts!(ByType);
impl_type_counts!(SpendableType);
