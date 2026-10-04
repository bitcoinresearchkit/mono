use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, Difficulty, Epoch, PartsPerMillionSigned32};
use bitview_transforms::{BlocksToDays, DifficultyToHashrate};
use bitview_vecs::{LazyFixedRatioPerBlock, LazyPerBlock, Resolutions};
use brk_types::{BLOCKS_PER_DIFF_EPOCHS, Height, Version};
use vecdb::{Ident, IndexVec, ReadOnlyClone};

use super::Vecs;

fn blocks_left_to_retarget(height: Height) -> Count {
    Count::from(u64::from(height.left_before_next_diff_adj()))
}

fn difficulty_adjustment(
    current: Difficulty,
    previous: Option<Difficulty>,
) -> PartsPerMillionSigned32 {
    match previous {
        Some(previous) => PartsPerMillionSigned32::from(*current / *previous - 1.0),
        None => PartsPerMillionSigned32::from(f32::NAN),
    }
}

impl Vecs {
    pub(crate) fn new(version: Version, indexer: &Indexer, mappings: &MappingsVecs) -> Self {
        let v2 = Version::TWO;

        let difficulty_source = indexer.vecs().blocks.difficulty.read_only_clone();
        let hashrate = LazyPerBlock::from_height_source::<DifficultyToHashrate>(
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

        let days_to_retarget = LazyPerBlock::from_lazy::<BlocksToDays, Count>(
            "days_to_retarget",
            version + v2,
            &blocks_to_retarget,
        );

        Self {
            value: Resolutions::from_source("difficulty", &difficulty_source, version, mappings),
            hashrate,
            adjustment: LazyFixedRatioPerBlock::from_lookback_source(
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
