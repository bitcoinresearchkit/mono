use brk_types::{Halving, Height, StoredU32, Version};
use vecdb::{LazyVec, ReadOnlyClone, ReadableCloneableVec};

use super::Vecs;
use crate::{
    indexes,
    internal::{BlocksToDaysF32, CACHE_BUDGET, Identity, LazyPerBlock},
};

fn blocks_left_to_halving(height: Height, _: Halving) -> StoredU32 {
    StoredU32::from(height.left_before_next_halving())
}

impl Vecs {
    pub(crate) fn new(version: Version, indexes: &indexes::Vecs) -> Self {
        let v2 = Version::TWO;

        let epoch_source = CACHE_BUDGET.wrap(indexes.height.halving.read_only_clone());
        let epoch = LazyPerBlock::from_height_source::<Identity<Halving>>(
            "halving_epoch",
            version,
            epoch_source,
            indexes,
        );
        let blocks_to_halving_source = LazyVec::init(
            "blocks_to_halving_source",
            version + v2,
            indexes.height.halving.read_only_boxed_clone(),
            blocks_left_to_halving,
        );
        let blocks_to_halving_source = CACHE_BUDGET.wrap(blocks_to_halving_source);
        let blocks_to_halving = LazyPerBlock::from_height_source::<Identity<StoredU32>>(
            "blocks_to_halving",
            version + v2,
            blocks_to_halving_source,
            indexes,
        );

        let days_to_halving = LazyPerBlock::from_lazy::<BlocksToDaysF32, StoredU32>(
            "days_to_halving",
            version + v2,
            &blocks_to_halving,
        );

        Self {
            epoch,
            blocks_to_halving,
            days_to_halving,
        }
    }
}
