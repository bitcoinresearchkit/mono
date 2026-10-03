use bitview_collections::Windows;
use bitview_compute::{ComputedVecValue, NumericValue};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, PerBlock};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RollingWindows<T, M: StorageMode = Rw>(pub Windows<PerBlock<T, M>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> RollingWindows<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(Windows::try_from_fn(|suffix| {
            PerBlock::import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }
}
