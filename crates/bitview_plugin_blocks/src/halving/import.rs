use bitview_compute::{BlocksToDaysF32, CACHE_BUDGET, Identity, LazyPerBlock};
use brk_types::{Halving, Height, StoredU32, Version};
use vecdb::{LazyVec, ReadOnlyClone, ReadableCloneableVec};

use super::Vecs;

fn blocks_left_to_halving(height: Height, _: Halving) -> StoredU32 {
    StoredU32::from(height.left_before_next_halving())
}

pub trait Import {
    fn new(version: Version, mappings: &bitview_plugin_mappings::Vecs) -> Self;
}

impl Import for Vecs {
    fn new(version: Version, mappings: &bitview_plugin_mappings::Vecs) -> Self {
        let v2 = Version::TWO;

        let epoch_source = CACHE_BUDGET.wrap(mappings.height.halving.read_only_clone());
        let epoch = LazyPerBlock::from_height_source::<Identity<Halving>>(
            "halving_epoch",
            version,
            epoch_source,
            mappings,
        );
        let blocks_to_halving_source = LazyVec::init(
            "blocks_to_halving_source",
            version + v2,
            mappings.height.halving.read_only_boxed_clone(),
            blocks_left_to_halving,
        );
        let blocks_to_halving_source = CACHE_BUDGET.wrap(blocks_to_halving_source);
        let blocks_to_halving = LazyPerBlock::from_height_source::<Identity<StoredU32>>(
            "blocks_to_halving",
            version + v2,
            blocks_to_halving_source,
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

#[cfg(test)]
mod tests {
    use brk_types::{Halving, Height, StoredU32};
    use vecdb::UnaryTransform;

    use super::blocks_left_to_halving;
    use bitview_compute::BlocksToDaysF32;

    #[test]
    fn formulas_match_public_halving_series_contracts() {
        for (height, epoch, remaining) in [
            (0_u32, 0_u8, 210_000_u32),
            (209_999, 0, 1),
            (210_000, 1, 210_000),
            (419_999, 1, 1),
            (420_000, 2, 210_000),
        ] {
            let height = Height::from(height);
            assert_eq!(Halving::from(height), Halving::new(epoch));
            assert_eq!(
                blocks_left_to_halving(height, Halving::new(epoch)),
                StoredU32::new(remaining)
            );
        }

        let days = BlocksToDaysF32::apply(StoredU32::new(210_000));
        assert_eq!(*days, 210_000.0_f32 / 144.0);
    }
}
