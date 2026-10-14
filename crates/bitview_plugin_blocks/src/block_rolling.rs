use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::{Compact, Traversable};
use bitview_vecs::{IndexSources, LazyWindowStartVec, PerBlockRolling, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, LazyVec, PcoVec, PcoVecValue, Rw, StorageMode};

/// A per-block quantity the indexer stores: its value per block and its rolling statistics.
#[derive(Deref, DerefMut, Traversable)]
pub struct BlockRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity,
{
    /// Value for the represented block.
    pub block: LazyVec<Height, T::Sum, Height, T>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub rolling: PerBlockRolling<T, M>,
}

impl<T> BlockRolling<T>
where
    T: NumericValue + JsonSchema + Quantity + PcoVecValue,
{
    /// The per-block series is the indexer's `source`, in the quantity's wide type.
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        source: &Compact<PcoVec<Height, T>, T::Sum>,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            block: source.view().clone(),
            rolling: PerBlockRolling::import(db, name, version, indexes, window_starts)?,
        })
    }
}
