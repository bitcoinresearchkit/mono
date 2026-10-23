use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;

use super::{BlockRolling, DifficultyVecs, Fullness, HalvingVecs, IntervalVecs, STORAGE, Vecs, count};

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

        let weight = BlockRolling::import(
            &db,
            "block_weight",
            version,
            &blocks.weight,
            mappings,
            &window_starts,
        )?;
        let fullness = Fullness::new(
            version,
            &*blocks.weight,
            weight.avg(),
            weight.distribution(),
            mappings,
        );
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
            weight,
            fullness,
            difficulty: DifficultyVecs::new(version, indexer, mappings),
            halving: HalvingVecs::new(version, mappings),
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
