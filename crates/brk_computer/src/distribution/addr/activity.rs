//! Per-block address activity, split by address type.

use brk_cohort::{AddrTypeId, ByAddrType};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{StoredU32, StoredU64, Version};
use derive_more::{Deref, DerefMut};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, ColumnId, Database, ReadOnlyClone, Rw, StorageMode};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling, LazyPerBlockCumulativeAverage,
        StoredU64ToStoredU32, Windows, WithAddrTypes,
    },
};

/// Per-block activity counts, reset after every block.
#[derive(Debug, Default, Clone)]
pub struct BlockActivityCounts {
    pub reactivated: u32,
    pub sending: u32,
    pub receiving: u32,
    pub bidirectional: u32,
}

impl BlockActivityCounts {
    #[inline]
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    #[inline(always)]
    fn active(&self) -> u32 {
        debug_assert!(self.bidirectional <= self.sending.min(self.receiving));
        self.sending + self.receiving - self.bidirectional
    }
}

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
    fn row(
        &self,
        value: impl Fn(&BlockActivityCounts) -> u32,
    ) -> <AddrTypeId as ColumnId>::Row<StoredU64> {
        AddrTypeId::from_fn(|column| {
            StoredU64::from(StoredU32::from(value(column.select(&self.0))))
        })
    }
}

#[derive(Clone, Traversable)]
pub struct ActivityCountVecs {
    pub reactivated: LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>,
    pub sending: LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>,
    pub receiving: LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>,
    pub bidirectional: LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>,
    pub active: LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>,
}

/// Five metric-first cumulative matrices with cohort-first public views.
#[derive(Deref, DerefMut, Traversable)]
pub struct AddrActivityVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: WithAddrTypes<ActivityCountVecs>,

    #[traversable(hidden)]
    cumulative_reactivated: ColumnarPerBlockCumulativeRolling<StoredU64, AddrTypeId, (), M>,
    #[traversable(hidden)]
    cumulative_sending: ColumnarPerBlockCumulativeRolling<StoredU64, AddrTypeId, (), M>,
    #[traversable(hidden)]
    cumulative_receiving: ColumnarPerBlockCumulativeRolling<StoredU64, AddrTypeId, (), M>,
    #[traversable(hidden)]
    cumulative_bidirectional: ColumnarPerBlockCumulativeRolling<StoredU64, AddrTypeId, (), M>,
    #[traversable(hidden)]
    cumulative_active: ColumnarPerBlockCumulativeRolling<StoredU64, AddrTypeId, (), M>,
}

impl AddrActivityVecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let cumulative_version = version + Version::TWO;
        let cumulative_reactivated = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "reactivated_addrs_by_type_cumulative",
            cumulative_version,
            |_| (),
        )?;
        let cumulative_sending = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "sending_addrs_by_type_cumulative",
            cumulative_version,
            |_| (),
        )?;
        let cumulative_receiving = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "receiving_addrs_by_type_cumulative",
            cumulative_version,
            |_| (),
        )?;
        let cumulative_bidirectional = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "bidirectional_addrs_by_type_cumulative",
            cumulative_version,
            |_| (),
        )?;
        let cumulative_active = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "active_addrs_by_type_cumulative",
            cumulative_version,
            |_| (),
        )?;

        let reactivated = WithAddrTypes::from_columnar_cumulative_average_source(
            "reactivated_addrs",
            version,
            &cumulative_reactivated.cumulative.read_only_clone(),
            indexes,
            cached_starts,
        );
        let sending = WithAddrTypes::from_columnar_cumulative_average_source(
            "sending_addrs",
            version,
            &cumulative_sending.cumulative.read_only_clone(),
            indexes,
            cached_starts,
        );
        let receiving = WithAddrTypes::from_columnar_cumulative_average_source(
            "receiving_addrs",
            version,
            &cumulative_receiving.cumulative.read_only_clone(),
            indexes,
            cached_starts,
        );
        let bidirectional = WithAddrTypes::from_columnar_cumulative_average_source(
            "bidirectional_addrs",
            version,
            &cumulative_bidirectional.cumulative.read_only_clone(),
            indexes,
            cached_starts,
        );
        let active = WithAddrTypes::from_columnar_cumulative_average_source(
            "active_addrs",
            version,
            &cumulative_active.cumulative.read_only_clone(),
            indexes,
            cached_starts,
        );

        let by_addr_type = AddrTypeId::series(|column, _| ActivityCountVecs {
            reactivated: column.select(&reactivated.by_addr_type).clone(),
            sending: column.select(&sending.by_addr_type).clone(),
            receiving: column.select(&receiving.by_addr_type).clone(),
            bidirectional: column.select(&bidirectional.by_addr_type).clone(),
            active: column.select(&active.by_addr_type).clone(),
        });
        let series = WithAddrTypes {
            all: ActivityCountVecs {
                reactivated: reactivated.all,
                sending: sending.all,
                receiving: receiving.all,
                bidirectional: bidirectional.all,
                active: active.all,
            },
            by_addr_type,
        };

        Ok(Self {
            series,
            cumulative_reactivated,
            cumulative_sending,
            cumulative_receiving,
            cumulative_bidirectional,
            cumulative_active,
        })
    }

    pub(crate) fn min_stateful_len(&self) -> usize {
        [
            self.cumulative_reactivated.cumulative.len(),
            self.cumulative_sending.cumulative.len(),
            self.cumulative_receiving.cumulative.len(),
            self.cumulative_bidirectional.cumulative.len(),
            self.cumulative_active.cumulative.len(),
        ]
        .into_iter()
        .min()
        .unwrap_or_default()
    }

    pub(crate) fn par_iter_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            self.cumulative_reactivated.stored_mut(),
            self.cumulative_sending.stored_mut(),
            self.cumulative_receiving.stored_mut(),
            self.cumulative_bidirectional.stored_mut(),
            self.cumulative_active.stored_mut(),
        ]
        .into_par_iter()
    }

    pub(crate) fn reset_height(&mut self) -> Result<()> {
        self.cumulative_reactivated.reset()?;
        self.cumulative_sending.reset()?;
        self.cumulative_receiving.reset()?;
        self.cumulative_bidirectional.reset()?;
        self.cumulative_active.reset()?;
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn push_height(&mut self, counts: &AddrTypeToActivityCounts) {
        self.cumulative_reactivated
            .push_block(counts.row(|counts| counts.reactivated));
        self.cumulative_sending
            .push_block(counts.row(|counts| counts.sending));
        self.cumulative_receiving
            .push_block(counts.row(|counts| counts.receiving));
        self.cumulative_bidirectional
            .push_block(counts.row(|counts| counts.bidirectional));
        self.cumulative_active
            .push_block(counts.row(BlockActivityCounts::active));
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
