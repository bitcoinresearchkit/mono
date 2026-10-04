use bitview_collections::Windows;
use bitview_compute::FixedRatio;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{FixedRatioPerBlock, IndexSources};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct FixedRatioRollingWindows<B: FixedRatio, M: StorageMode = Rw>(
    pub Windows<FixedRatioPerBlock<B, M>>,
);

impl<B: FixedRatio> FixedRatioRollingWindows<B> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(Windows::try_from_fn(|suffix| {
            FixedRatioPerBlock::import(
                db,
                &format!("{name}_{suffix}"),
                version + Version::ONE,
                indexes,
            )
        })?))
    }
}
