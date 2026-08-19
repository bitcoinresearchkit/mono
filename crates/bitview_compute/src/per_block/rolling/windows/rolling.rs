use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

use crate::{ComputedVecValue, NumericValue, PerBlock, Windows};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RollingWindows<T, M: StorageMode = Rw>(pub Windows<PerBlock<T, M>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> RollingWindows<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self(Windows::try_from_fn(|suffix| {
            PerBlock::forced_import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }
}
