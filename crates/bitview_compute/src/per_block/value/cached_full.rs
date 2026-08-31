use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    BinaryTransform, CachedBoxedVec, Database, ReadOnlyClone, ReadableVec, Rw, StorageMode,
    VecIndex, VecValue,
};

use crate::{
    CachedValuePerBlock, CachedWindowStartVec, LazyRollingAvgsAmountFromHeight,
    LazyRollingSumsAmountFromHeight, LazyValueBlock, RollingDistributionValuePerBlock, SatsToCents,
    WindowStarts, Windows,
};

#[derive(Traversable)]
pub struct CachedValuePerBlockFull<M: StorageMode = Rw> {
    /// Value for the represented block. At time-period indexes, the value is
    /// taken from the period's final block.
    pub block: LazyValueBlock,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: CachedValuePerBlock<M>,
    pub sum: LazyRollingSumsAmountFromHeight,
    pub average: LazyRollingAvgsAmountFromHeight,
    #[traversable(flatten)]
    pub distribution: RollingDistributionValuePerBlock<M>,
}

const VERSION: Version = Version::TWO;

impl CachedValuePerBlockFull {
    pub fn cached_cumulative_sats(&self) -> CachedBoxedVec<Height, Sats> {
        self.cumulative.sats.height.read_only_cached_boxed_clone()
    }

    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let full_version = version + VERSION;
        let rolling_version = full_version + Version::TWO;
        let cumulative_version = rolling_version + Version::ONE;
        let cumulative = CachedValuePerBlock::forced_import(
            db,
            &format!("{name}_cumulative"),
            cumulative_version,
            indexes,
        )?;
        let cumulative_sats = cumulative.sats.height.read_only_clone();
        let block = LazyValueBlock::from_cumulative_sources(
            name,
            cumulative_version,
            &cumulative_sats,
            &cumulative.cents.height,
        );
        let sum = LazyRollingSumsAmountFromHeight::new(
            &format!("{name}_sum"),
            rolling_version,
            &cumulative_sats,
            &cumulative.cents.height,
            cached_starts,
            indexes,
        );
        let average = LazyRollingAvgsAmountFromHeight::new(
            &format!("{name}_average"),
            rolling_version,
            &cumulative_sats,
            &cumulative.cents.height,
            cached_starts,
            indexes,
        );
        let distribution =
            RollingDistributionValuePerBlock::forced_import(db, name, full_version, indexes)?;

        Ok(Self {
            block,
            cumulative,
            sum,
            average,
            distribution,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_from_indexes<A, B>(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        price_cents: &impl ReadableVec<Height, Cents>,
        first_indexes: &impl ReadableVec<Height, A>,
        indexes_count: &impl ReadableVec<Height, B>,
        source: &impl ReadableVec<A, Sats>,
        exit: &Exit,
    ) -> Result<()>
    where
        A: VecIndex + VecValue,
        B: VecValue,
        usize: From<B>,
    {
        self.cumulative.compute_sats_from_indexes(
            max_from,
            first_indexes,
            indexes_count,
            source,
            exit,
        )?;

        self.cumulative
            .cents
            .height
            .compute_cumulative_transformed_binary(
                max_from,
                &self.block.sats,
                price_cents,
                SatsToCents::apply,
                exit,
            )?;

        self.compute_distribution(max_from, windows, exit)
    }

    fn compute_distribution(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        exit: &Exit,
    ) -> Result<()> {
        self.distribution
            .compute(max_from, windows, &self.block.sats, &self.block.cents, exit)?;

        Ok(())
    }
}
