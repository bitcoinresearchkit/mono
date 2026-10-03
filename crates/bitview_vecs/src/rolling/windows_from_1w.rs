use bitview_collections::WindowsFrom1w;
use bitview_compute::NumericValue;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

use crate::{IndexSources, PerBlock};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RollingWindowsFrom1w<T: NumericValue + JsonSchema, M: StorageMode = Rw>(
    pub WindowsFrom1w<PerBlock<T, M>>,
);

impl<T: NumericValue + JsonSchema> RollingWindowsFrom1w<T> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(WindowsFrom1w::try_from_fn(|suffix| {
            PerBlock::import(
                db,
                &format!("{name}_{suffix}"),
                version + Version::ONE,
                indexes,
            )
        })?))
    }
}
