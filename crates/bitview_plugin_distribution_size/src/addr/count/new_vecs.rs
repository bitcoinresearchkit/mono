use bitview_cohort::WithAddrTypes;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeRolling, LazyWindowStartVec};
use brk_types::{StoredU64, Version};
use derive_more::{Deref, DerefMut};

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
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        Self(WithAddrTypes {
            all: LazyPerBlockCumulativeRolling::from_lazy_source(
                "new_addr_count",
                version,
                &total.all,
                window_starts,
                mappings,
            ),
            by_addr_type: total.by_addr_type.map_with_name(|name, total| {
                LazyPerBlockCumulativeRolling::from_lazy_source(
                    &format!("{name}_new_addr_count"),
                    version,
                    total,
                    window_starts,
                    mappings,
                )
            }),
        })
    }
}
