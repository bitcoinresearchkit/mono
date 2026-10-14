use bitview_compute::{ComputedVecValue, NumericValue};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::TxIndex;
use schemars::JsonSchema;
use vecdb::{Database, EagerVec, ImportableVec, PcoVec, Rw, StorageMode, Version};

use crate::{IndexSources, TxDistribution};

/// A stored per-transaction value (`tx`) beside its per-block distribution.
#[derive(Traversable)]
pub struct PerTxDistribution<T, M: StorageMode = Rw>
where
    T: ComputedVecValue + PartialOrd + JsonSchema,
{
    pub tx: M::Stored<EagerVec<PcoVec<TxIndex, T>>>,
    #[traversable(flatten)]
    pub distribution: TxDistribution<T, M>,
}

impl<T> PerTxDistribution<T>
where
    T: NumericValue + JsonSchema + Ord,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self {
            tx: EagerVec::import(db, name, version)?,
            distribution: TxDistribution::import(db, name, version, indexes)?,
        })
    }
}
