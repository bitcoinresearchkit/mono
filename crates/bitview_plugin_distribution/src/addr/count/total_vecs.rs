use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::AddrTypeId;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, Database, Exit, ReadOnlyClone, Rw, StorageMode};

use super::AddrCountsVecs;

/// Total address count (global + per-type) with all derived indexes.
#[derive(Deref, DerefMut, Traversable)]
pub struct TotalAddrCountVecs<M: StorageMode = Rw>(#[traversable(flatten)] pub AddrCountsVecs<M>);

impl TotalAddrCountVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self(AddrCountsVecs::forced_import(
            db,
            "total_addr_count",
            version,
            indexes,
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
        let addr_count = addr_count.height.read_only_clone();
        let empty_addr_count = empty_addr_count.height.read_only_clone();
        self.height.compute_transform2(
            max_from,
            &addr_count,
            &empty_addr_count,
            |(height, addr_count, empty_addr_count, ..)| {
                let total = AddrTypeId::from_fn(|column| {
                    *column.get(&addr_count) + *column.get(&empty_addr_count)
                });
                (height, total)
            },
            exit,
        )?;

        Ok(())
    }
}
