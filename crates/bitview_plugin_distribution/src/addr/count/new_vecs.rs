use bitview_traversable::Traversable;
use brk_types::{StoredU64, Version};
use derive_more::{Deref, DerefMut};

use bitview_compute::{
    CachedWindowStartVec, LazyPerBlockCumulativeRolling, Windows, WithAddrTypes,
};

use super::TotalAddrCountVecs;

/// New address count per block (global + per-type).
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct NewAddrCountVecs(
    #[traversable(flatten)] pub WithAddrTypes<LazyPerBlockCumulativeRolling<StoredU64>>,
);

impl NewAddrCountVecs {
    pub fn new(
        version: Version,
        total: &TotalAddrCountVecs,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        Self(WithAddrTypes {
            all: LazyPerBlockCumulativeRolling::from_lazy_source(
                "new_addr_count",
                version,
                &total.all,
                cached_starts,
                indexes,
            ),
            by_addr_type: total.by_addr_type.map_with_name(|name, total| {
                LazyPerBlockCumulativeRolling::from_column_source(
                    &format!("{name}_new_addr_count"),
                    version,
                    total,
                    cached_starts,
                    indexes,
                )
            }),
        })
    }
}
