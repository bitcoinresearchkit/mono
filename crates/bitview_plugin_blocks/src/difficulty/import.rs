use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{BlocksToDaysF32, DifficultyToHashF64};
use bitview_vecs::{LazyPerBlock, LazyPercentPerBlock, Resolutions};
use brk_types::{
    BLOCKS_PER_DIFF_EPOCHS, Epoch, Height, PartsPerMillionSigned32, StoredF64, StoredU32, Version,
};
use vecdb::{Ident, IndexVec, ReadOnlyClone};

use super::Vecs;

fn blocks_left_to_retarget(height: Height) -> StoredU32 {
    StoredU32::from(height.left_before_next_diff_adj())
}

fn difficulty_adjustment(
    current: StoredF64,
    previous: Option<StoredF64>,
) -> PartsPerMillionSigned32 {
    match previous {
        Some(previous) => {
            PartsPerMillionSigned32::from((f32::from(current) / f32::from(previous)) - 1.0)
        }
        None => PartsPerMillionSigned32::from(f32::NAN),
    }
}

impl Vecs {
    pub(crate) fn new(version: Version, indexer: &Indexer, mappings: &MappingsVecs) -> Self {
        let v2 = Version::TWO;

        let difficulty_source = indexer.vecs().blocks.difficulty.read_only_clone();
        let hashrate = LazyPerBlock::from_height_source::<DifficultyToHashF64>(
            "difficulty_hashrate",
            version,
            &difficulty_source,
            mappings,
        );

        let epoch_source = IndexVec::new(
            "difficulty_epoch_source",
            Version::ZERO,
            mappings.height.epoch.read_only_clone(),
            Epoch::from,
        );
        let epoch = LazyPerBlock::from_height_source::<Ident>(
            "difficulty_epoch",
            version,
            &epoch_source,
            mappings,
        );
        let blocks_to_retarget_source = IndexVec::new(
            "blocks_to_retarget_source",
            version + v2,
            mappings.height.epoch.read_only_clone(),
            blocks_left_to_retarget,
        );
        let blocks_to_retarget = LazyPerBlock::from_height_source::<Ident>(
            "blocks_to_retarget",
            version + v2,
            &blocks_to_retarget_source,
            mappings,
        );

        let days_to_retarget = LazyPerBlock::from_lazy::<BlocksToDaysF32, StoredU32>(
            "days_to_retarget",
            version + v2,
            &blocks_to_retarget,
        );

        Self {
            value: Resolutions::from_source("difficulty", &difficulty_source, version, mappings),
            hashrate,
            adjustment: LazyPercentPerBlock::from_lookback_source(
                "difficulty_adjustment",
                version + Version::ONE,
                &difficulty_source,
                BLOCKS_PER_DIFF_EPOCHS as usize,
                difficulty_adjustment,
                mappings,
            ),
            epoch,
            blocks_to_retarget,
            days_to_retarget,
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Height, PartsPerMillionSigned32, StoredF64, StoredU32};
    use vecdb::UnaryTransform;

    use super::{blocks_left_to_retarget, difficulty_adjustment};
    use bitview_transforms::{BlocksToDaysF32, DifficultyToHashF64};

    #[test]
    fn formulas_match_public_difficulty_series_contracts() {
        for (height, expected) in [(0_u32, 2_016_u32), (1, 2_015), (2_015, 1), (2_016, 2_016)] {
            assert_eq!(
                blocks_left_to_retarget(Height::from(height)),
                StoredU32::new(expected)
            );
        }

        assert!(difficulty_adjustment(StoredF64::from(1.0), None).is_nan());
        assert_eq!(
            difficulty_adjustment(StoredF64::from(110.0), Some(StoredF64::from(100.0))),
            PartsPerMillionSigned32::from(0.1)
        );

        let hashrate = DifficultyToHashF64::apply(StoredF64::from(1.0));
        assert_eq!(*hashrate, 4_294_967_296.0 / 600.0);

        let days = BlocksToDaysF32::apply(StoredU32::new(2_016));
        assert_eq!(*days, 14.0);
    }
}
