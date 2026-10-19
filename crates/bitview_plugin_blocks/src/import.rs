use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::LazyPercentVec;
use brk_error::Result;
use brk_types::{Height, Weight, Weight32};

use super::{BlockRolling, DifficultyVecs, HalvingVecs, IntervalVecs, STORAGE, Vecs, count};

fn block_fullness(_: Height, weight: Weight32) -> PartsPerMillion32 {
    PartsPerMillion32::from(Weight::from(weight).fullness())
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        indexer: &Indexer,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 1_000_000)?;
        let version = STORAGE.schema_version();
        let window_starts = mappings.lookback.window_starts();
        let blocks = &indexer.vecs().blocks;

        let this = Self {
            count: count::block_count(version, indexer, mappings, &window_starts),
            interval: IntervalVecs::import(&db, version, mappings, &window_starts)?,
            size: BlockRolling::import(
                &db,
                "block_size",
                version,
                &blocks.size,
                mappings,
                &window_starts,
            )?,
            weight: BlockRolling::import(
                &db,
                "block_weight",
                version,
                &blocks.weight,
                mappings,
                &window_starts,
            )?,
            fullness: LazyPercentVec::from_indexed_source(
                "block_fullness",
                version,
                &*blocks.weight,
                block_fullness,
            ),
            difficulty: DifficultyVecs::new(version, indexer, mappings),
            halving: HalvingVecs::new(version, mappings),
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
