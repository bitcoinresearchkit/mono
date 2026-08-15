use brk_indexer::Indexer;
use brk_types::{Epoch, Height, PartsPerMillionSigned32, StoredF64, StoredU32, Version};
use vecdb::{LazyVec, ReadOnlyClone, ReadableCloneableVec};

use super::Vecs;
use crate::{
    indexes,
    internal::{
        BlocksToDaysF32, CACHE_BUDGET, DifficultyToHashF64, Identity, LazyPerBlock,
        LazyPercentPerBlock, Resolutions,
    },
};

const DIFFICULTY_ADJUSTMENT_LOOKBACK: usize = 2016;

fn blocks_left_to_retarget(height: Height, _: Epoch) -> StoredU32 {
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
    pub(crate) fn new(version: Version, indexer: &Indexer, indexes: &indexes::Vecs) -> Self {
        let v2 = Version::TWO;

        let difficulty_source =
            CACHE_BUDGET.wrap(indexer.vecs().blocks.difficulty.read_only_clone());
        let hashrate = LazyPerBlock::from_height_source::<DifficultyToHashF64>(
            "difficulty_hashrate",
            version,
            difficulty_source.clone(),
            indexes,
        );

        let epoch_source = CACHE_BUDGET.wrap(indexes.height.epoch.read_only_clone());
        let epoch = LazyPerBlock::from_height_source::<Identity<Epoch>>(
            "difficulty_epoch",
            version,
            epoch_source,
            indexes,
        );
        let blocks_to_retarget_source = LazyVec::init(
            "blocks_to_retarget_source",
            version + v2,
            indexes.height.epoch.read_only_boxed_clone(),
            blocks_left_to_retarget,
        );
        let blocks_to_retarget_source = CACHE_BUDGET.wrap(blocks_to_retarget_source);
        let blocks_to_retarget = LazyPerBlock::from_height_source::<Identity<StoredU32>>(
            "blocks_to_retarget",
            version + v2,
            blocks_to_retarget_source,
            indexes,
        );

        let days_to_retarget = LazyPerBlock::from_lazy::<BlocksToDaysF32, StoredU32>(
            "days_to_retarget",
            version + v2,
            &blocks_to_retarget,
        );

        Self {
            value: Resolutions::from_height_source(
                "difficulty",
                difficulty_source,
                version,
                indexes,
            ),
            hashrate,
            adjustment: LazyPercentPerBlock::from_lookback_source(
                "difficulty_adjustment",
                version + Version::ONE,
                &indexer.vecs().blocks.difficulty,
                DIFFICULTY_ADJUSTMENT_LOOKBACK,
                difficulty_adjustment,
                indexes,
            ),
            epoch,
            blocks_to_retarget,
            days_to_retarget,
        }
    }
}
