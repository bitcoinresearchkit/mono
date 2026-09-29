use bitview_cohort::WithAddrTypes;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyRollingDeltasFromHeight, LazyWindowStartVec};
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use derive_more::{Deref, DerefMut};

use super::AddrCountsVecs;

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct DeltaVecs(
    #[traversable(flatten)]
    pub  WithAddrTypes<LazyRollingDeltasFromHeight<StoredU64, StoredI64, PartsPerMillionSigned64>>,
);

impl DeltaVecs {
    pub fn new(
        version: Version,
        addr_count: &AddrCountsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        mappings: &MappingsVecs,
    ) -> Self {
        let version = version + Version::new(3);

        let all = LazyRollingDeltasFromHeight::new(
            "addr_count",
            version,
            &addr_count.all.height,
            window_starts,
            mappings,
        );

        let by_addr_type = addr_count.by_addr_type.map_with_name(|name, addr| {
            LazyRollingDeltasFromHeight::new(
                &format!("{name}_addr_count"),
                version,
                &addr.height,
                window_starts,
                mappings,
            )
        });

        Self(WithAddrTypes { all, by_addr_type })
    }
}
