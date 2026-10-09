use bitview_collections::Windows;
use bitview_compute::FixedRatio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, RatioPerBlock};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RatioRollingWindows<R: FixedRatio, M: StorageMode = Rw>(
    pub Windows<RatioPerBlock<R, M>>,
);

impl<R: FixedRatio> RatioRollingWindows<R> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(Windows::try_from_fn(|suffix| {
            RatioPerBlock::import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }
}
