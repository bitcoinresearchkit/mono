use bitview_compute::{NumericValue, Quantity};
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockRolling;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{LazyVec, ReadableCloneableVec, Rw, StorageMode};

/// A per-block quantity the indexer stores: its value per block and its rolling statistics.
#[derive(Deref, DerefMut, Traversable)]
pub struct BlockRolling<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema + Quantity,
{
    /// Value for the represented block.
    pub block: LazyVec<Height, T, Height, T>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub rolling: PerBlockRolling<T, M>,
}

impl<T: NumericValue + JsonSchema + Quantity> BlockRolling<T> {
    /// `name` is the per-block series (`block_size`), a view of the indexer's `source`.
    pub fn new(
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, T>,
        rolling: PerBlockRolling<T>,
    ) -> Self {
        Self {
            block: LazyVec::init(name, version, source.read_only_boxed_clone(), |_, value| {
                value
            }),
            rolling,
        }
    }
}
