use bitview_collections::Windows;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::{
    IndexSources, RollingDistributionValuePerBlock, ValuePerBlockCumulativeRolling, WindowStarts,
};

#[derive(Deref, DerefMut, Traversable)]
pub struct ValuePerBlockFull<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    inner: ValuePerBlockCumulativeRolling<M>,
    #[traversable(flatten)]
    distribution: RollingDistributionValuePerBlock<M>,
}

const VERSION: Version = Version::TWO;

impl ValuePerBlockFull {
    pub fn compute_from(
        &mut self,
        max_from: Height,
        windows: &WindowStarts<'_>,
        price_cents: &impl ReadableVec<Height, Cents>,
        source: &impl ReadableVec<Height, Sats>,
        exit: &Exit,
    ) -> Result<()> {
        self.inner
            .compute_from(max_from, price_cents, source, |_, value| value, exit)?;
        self.distribution.compute(
            max_from,
            windows,
            &self.inner.block.sats,
            &self.inner.block.cents,
            exit,
        )
    }

    pub fn cumulative_sats_source(&self) -> &(impl ReadableCloneableVec<Height, Sats> + use<>) {
        self.cumulative.sats.resolutions.height_source()
    }

    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let full_version = version + VERSION;
        let inner = ValuePerBlockCumulativeRolling::forced_import(
            db,
            name,
            full_version,
            indexes,
            window_starts,
        )?;
        let distribution =
            RollingDistributionValuePerBlock::forced_import(db, name, full_version, indexes)?;

        Ok(Self {
            inner,
            distribution,
        })
    }
}
