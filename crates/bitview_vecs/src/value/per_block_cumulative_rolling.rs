use bitview_collections::Windows;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{Database, ReadableCloneableVec, Rw, StorageMode};

use crate::{IndexSources, RollingAmountTotals, ValuePerBlockCumulative};

#[derive(Deref, DerefMut, Traversable)]
pub struct ValuePerBlockCumulativeRolling<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    inner: ValuePerBlockCumulative<M>,
    #[traversable(flatten)]
    pub rolling: RollingAmountTotals,
}

const VERSION: Version = Version::TWO;

impl ValuePerBlockCumulativeRolling {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
    ) -> Result<Self> {
        let v = version + VERSION;

        let inner = ValuePerBlockCumulative::import(db, name, v, indexes)?;
        let rolling = RollingAmountTotals::new(
            name,
            v,
            inner.cumulative.sats.resolutions.height_source(),
            inner.cumulative.cents.resolutions.height_source(),
            window_starts,
            indexes,
        );

        Ok(Self { inner, rolling })
    }
}
