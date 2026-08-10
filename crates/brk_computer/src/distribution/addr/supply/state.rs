use brk_cohort::{AddrTypeId, ByAddrType};
use brk_types::{Height, OutputType, Sats};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnId, ReadableVec};

use super::vecs::AddrSupplyVecs;

/// Per-addr-type running-total of a supply category (sats). Shared across
/// predicate-based supply categories (exposed, reused, respent).
#[derive(Debug, Default, Deref, DerefMut)]
pub struct AddrTypeToSupply(ByAddrType<Sats>);

impl AddrTypeToSupply {
    #[inline]
    pub(crate) fn row(&self) -> <AddrTypeId as ColumnId>::Row<Sats> {
        AddrTypeId::from_fn(|column| *column.select(&self.0))
    }

    /// Apply a signed `after - before` delta to the slot for `output_type`.
    /// Sats is unsigned, so branch on sign.
    #[inline]
    pub(crate) fn apply_delta(&mut self, output_type: OutputType, before: Sats, after: Sats) {
        let slot = self.get_mut_unwrap(output_type);
        if after >= before {
            *slot += after - before;
        } else {
            *slot -= before - after;
        }
    }
}

impl From<ByAddrType<Sats>> for AddrTypeToSupply {
    #[inline]
    fn from(value: ByAddrType<Sats>) -> Self {
        Self(value)
    }
}

impl From<(&AddrSupplyVecs, Height)> for AddrTypeToSupply {
    #[inline]
    fn from((vecs, starting_height): (&AddrSupplyVecs, Height)) -> Self {
        let Some(prev_height) = starting_height.decremented() else {
            return Self::default();
        };
        vecs.by_addr_type
            .map_with_name(|_, v| v.sats.height.collect_one(prev_height).unwrap())
            .into()
    }
}
