use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::BlocksToDaysF32;
use bitview_vecs::LazyPerBlock;
use brk_types::{Halving, Height, StoredU32, Version};
use vecdb::{Ident, IndexVec, ReadOnlyClone};

use super::Vecs;

fn blocks_left_to_halving(height: Height) -> StoredU32 {
    StoredU32::from(height.left_before_next_halving())
}

impl Vecs {
    pub fn new(version: Version, mappings: &MappingsVecs) -> Self {
        let v2 = Version::TWO;

        let epoch_source = IndexVec::new(
            "halving_epoch_source",
            Version::ZERO,
            mappings.height.halving.read_only_clone(),
            Halving::from,
        );
        let epoch = LazyPerBlock::from_height_source::<Ident>(
            "halving_epoch",
            version,
            &epoch_source,
            mappings,
        );
        let blocks_to_halving_source = IndexVec::new(
            "blocks_to_halving_source",
            version + v2,
            mappings.height.halving.read_only_clone(),
            blocks_left_to_halving,
        );
        let blocks_to_halving = LazyPerBlock::from_height_source::<Ident>(
            "blocks_to_halving",
            version + v2,
            &blocks_to_halving_source,
            mappings,
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
