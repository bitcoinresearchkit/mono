use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, Difficulty, DifficultyEpoch, PartsPerMillionSigned32};
use bitview_vecs::{
    LazyPerBlock, LazyPercentPerBlock, Resolutions,
    ops::{BlocksToDays, DifficultyToHashrate, RelativeChange},
};
use brk_types::{BLOCKS_PER_DIFF_EPOCH, Height, Version};
use vecdb::{Ident, IndexVec, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec};

use super::Vecs;

fn blocks_left_to_retarget(height: Height) -> Count {
    Count::from(u64::from(height.left_before_next_diff_adj()))
}

/// The first block of the window, a retarget period long, that ends at `height`: the value
/// before it is the one `BLOCKS_PER_DIFF_EPOCH` blocks earlier (none through the first period).
fn retarget_window_start(height: Height) -> Height {
    Height::from(usize::from(height).saturating_sub(BLOCKS_PER_DIFF_EPOCH as usize - 1))
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
            mappings.height.difficulty_epoch.read_only_clone(),
            DifficultyEpoch::from,
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
            mappings.height.difficulty_epoch.read_only_clone(),
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

        // The change from the difficulty before the window: zero (the default) through the first
        // period, where dividing by it leaves no value.
        let adjustment_source = LazyDeltaVec::<
            Height,
            Difficulty,
            PartsPerMillionSigned32,
            RelativeChange<PartsPerMillionSigned32>,
        >::new(
            "difficulty_adjustment_ppm_source",
            version + Version::ONE,
            difficulty_source.read_only_boxed_clone(),
            IndexVec::new(
                "retarget_window_start",
                Version::ZERO,
                difficulty_source.clone(),
                retarget_window_start,
            )
            .into_read_only_boxed(),
        );

        Self {
            value: Resolutions::from_source("difficulty", &difficulty_source, version, mappings),
            block_height: LazyVec::new(
                "difficulty",
                version,
                difficulty_source.read_only_boxed_clone(),
                |_, difficulty| difficulty,
            ),
            hashrate,
            adjustment: LazyPercentPerBlock::from_height_source(
                "difficulty_adjustment",
                version + Version::ONE,
                &adjustment_source,
                mappings,
            ),
            epoch,
            blocks_to_retarget,
            days_to_retarget,
        }
    }
}
