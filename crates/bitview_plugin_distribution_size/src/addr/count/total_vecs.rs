use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use super::AddrCountsVecs;

/// Total address count (global + per-type) with all derived mappings.
#[derive(Deref, DerefMut, Traversable)]
pub struct TotalAddrCountVecs<M: StorageMode = Rw>(#[traversable(flatten)] pub AddrCountsVecs<M>);

impl TotalAddrCountVecs {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        Ok(Self(AddrCountsVecs::forced_import(
            db,
            "total_addr_count",
            version,
            mappings,
        )?))
    }

    /// Eagerly compute total = addr_count + empty_addr_count.
    pub fn compute(
        &mut self,
        max_from: Height,
        addr_count: &AddrCountsVecs,
        empty_addr_count: &AddrCountsVecs,
        exit: &Exit,
    ) -> Result<()> {
        for ((target, funded), empty) in self
            .stored
            .iter_mut()
            .zip(addr_count.stored.iter())
            .zip(empty_addr_count.stored.iter())
        {
            target.compute_transform2(
                max_from,
                funded,
                empty,
                |(height, funded, empty, _)| (height, funded + empty),
                exit,
            )?;
        }

        Ok(())
    }
}
