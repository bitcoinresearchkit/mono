use brk_cohort::{AddrTypeId, ByAddrType};
use brk_types::{Height, StoredU64};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, ReadableVec};

use super::AddrCountsVecs;

/// Per-addr-type address-count running total. Shared runtime state across
/// funded / empty / exposed / reused / respent counters; paired with
/// [`AddrCountsVecs`] on disk.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct AddrTypeToAddrCount(ByAddrType<u64>);

impl AddrTypeToAddrCount {
    pub(crate) fn row(&self) -> <AddrTypeId as ColumnId>::Row<StoredU64> {
        AddrTypeId::from_fn(|id| StoredU64::from(*id.select(&self.0)))
    }
}

impl From<ByAddrType<u64>> for AddrTypeToAddrCount {
    #[inline]
    fn from(value: ByAddrType<u64>) -> Self {
        Self(value)
    }
}

impl From<(&AddrCountsVecs, Height)> for AddrTypeToAddrCount {
    #[inline]
    fn from((vecs, starting_height): (&AddrCountsVecs, Height)) -> Self {
        let Some(prev_height) = starting_height.decremented() else {
            return Self::default();
        };
        vecs.by_addr_type
            .map_with_name(|_, v| v.height.collect_one(prev_height).unwrap().into())
            .into()
    }
}
