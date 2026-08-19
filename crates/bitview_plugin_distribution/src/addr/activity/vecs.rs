use brk_error::Result;

use bitview_traversable::Traversable;
use brk_cohort::AddrTypeId;
use brk_types::{StoredU32, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, ReadOnlyClone, Rw, StorageMode};

use bitview_compute::{
    CachedWindowStartVec, ColumnarPerBlockCumulativeRolling, LazyPerBlockCumulativeAverage,
    StoredU64ToStoredU32, Windows, WithAddrTypes,
};

use super::{AddrTypeToActivityCounts, BlockActivityCounts};

#[derive(Traversable)]
pub struct AddrActivityVecs<M: StorageMode = Rw> {
    /// Distinct addresses that received after previously being empty in the
    /// represented block.
    pub reactivated:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses that sent bitcoin in the represented block.
    pub sending:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses that received bitcoin in the represented block.
    pub receiving:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses that both sent and received bitcoin in the
    /// represented block.
    pub bidirectional:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses active in the represented block: sending plus
    /// receiving minus bidirectional addresses.
    pub active:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,

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
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
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

        Ok(Self {
            reactivated,
            sending,
            receiving,
            bidirectional,
            active,
            cumulative_reactivated,
            cumulative_sending,
            cumulative_receiving,
            cumulative_bidirectional,
            cumulative_active,
        })
    }

    pub fn min_resume_len(&self) -> usize {
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

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        [
            self.cumulative_reactivated.stored_mut(),
            self.cumulative_sending.stored_mut(),
            self.cumulative_receiving.stored_mut(),
            self.cumulative_bidirectional.stored_mut(),
            self.cumulative_active.stored_mut(),
        ]
        .into_par_iter()
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.cumulative_reactivated.reset()?;
        self.cumulative_sending.reset()?;
        self.cumulative_receiving.reset()?;
        self.cumulative_bidirectional.reset()?;
        self.cumulative_active.reset()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_height(&mut self, counts: &AddrTypeToActivityCounts) {
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
