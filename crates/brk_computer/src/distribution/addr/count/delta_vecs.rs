use brk_traversable::Traversable;
use brk_types::{PartsPerMillionSigned64, StoredI64, StoredU64, Version};
use derive_more::{Deref, DerefMut};

use crate::{
    indexes,
    internal::{CachedWindowStartVec, LazyRollingDeltasFromHeight, Windows, WithAddrTypes},
};

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
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let version = version + Version::new(3);

        let all = LazyRollingDeltasFromHeight::new(
            "addr_count",
            version,
            &addr_count.all.height,
            cached_starts,
            indexes,
        );

        let by_addr_type = addr_count.by_addr_type.map_with_name(|name, addr| {
            LazyRollingDeltasFromHeight::new(
                &format!("{name}_addr_count"),
                version,
                &addr.height,
                cached_starts,
                indexes,
            )
        });

        Self(WithAddrTypes { all, by_addr_type })
    }
}
