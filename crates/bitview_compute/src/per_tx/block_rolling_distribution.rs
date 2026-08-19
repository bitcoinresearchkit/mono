use brk_error::Result;

use bitview_traversable::Traversable;
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode, Version};

use crate::{ComputedVecValue, NumericValue, PerBlockDistribution};

#[derive(Traversable)]
pub struct BlockRollingDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub _6b: PerBlockDistribution<T, M>,
}

impl<T> BlockRollingDistribution<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            _6b: PerBlockDistribution::forced_import(db, &format!("{name}_6b"), version, indexes)?,
        })
    }
}
