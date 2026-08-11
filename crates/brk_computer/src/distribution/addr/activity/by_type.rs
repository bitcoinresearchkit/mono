use brk_cohort::{AddrTypeId, ByAddrType};
use brk_types::{StoredU32, StoredU64};
use derive_more::{Deref, DerefMut};
use vecdb::ColumnId;

use super::BlockActivityCounts;

/// Activity counts accumulated during block processing for each address type.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct AddrTypeToActivityCounts(pub ByAddrType<BlockActivityCounts>);

impl AddrTypeToActivityCounts {
    pub(crate) fn reset(&mut self) {
        self.0.values_mut().for_each(BlockActivityCounts::reset);
    }

    pub(crate) fn active(&self) -> u32 {
        self.0.values().map(BlockActivityCounts::active).sum()
    }

    #[inline(always)]
    pub(super) fn row(
        &self,
        value: impl Fn(&BlockActivityCounts) -> u32,
    ) -> <AddrTypeId as ColumnId>::Row<StoredU64> {
        AddrTypeId::from_fn(|column| {
            StoredU64::from(StoredU32::from(value(column.select(&self.0))))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_count_sums_distinct_addresses_across_types() {
        let counts = AddrTypeToActivityCounts(ByAddrType {
            p2pkh: BlockActivityCounts {
                sending: 5,
                receiving: 4,
                bidirectional: 2,
                ..BlockActivityCounts::default()
            },
            p2tr: BlockActivityCounts {
                sending: 3,
                receiving: 2,
                bidirectional: 1,
                ..BlockActivityCounts::default()
            },
            ..ByAddrType::default()
        });

        assert_eq!(counts.active(), 11);
    }
}
