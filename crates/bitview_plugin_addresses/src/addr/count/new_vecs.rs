use bitview_cohort::WithAddrTypes;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlock, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, LazyWindowStartVec,
};
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};

use super::TotalAddrCountVecs;

/// New address count per block (global + per-type). Its running total is the
/// total address count.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct NewAddrCountVecs(#[traversable(flatten)] pub WithAddrTypes<NewAddrCount>);

#[derive(Clone, Traversable)]
pub struct NewAddrCount {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyPreviousDeltaVec<Height, Count>,
    pub sum: LazyRollingSumsFromHeight<Count>,
}

impl NewAddrCount {
    fn new(
        name: &str,
        version: Version,
        total: &LazyPerBlock<Count>,
        window_starts: &Windows<&LazyWindowStartVec>,
        mappings: &MappingsVecs,
    ) -> Self {
        Self {
            block: LazyPreviousDeltaVec::new(name, version, &total.height),
            sum: LazyRollingSumsFromHeight::new(
                &format!("{name}_sum"),
                version,
                &total.height,
                window_starts,
                mappings,
            ),
        }
    }
}

impl NewAddrCountVecs {
    pub fn new(
        version: Version,
        total: &TotalAddrCountVecs,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        Self(WithAddrTypes {
            all: NewAddrCount::new(
                "new_addr_count",
                version,
                &total.all,
                window_starts,
                mappings,
            ),
            by_addr_type: total.by_addr_type.map_with_name(|name, total| {
                NewAddrCount::new(
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
