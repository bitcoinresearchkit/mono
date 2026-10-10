use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::{LazyPercentVec, LazyWindowStartVec, PerBlockRolling};
use brk_error::Result;
use brk_types::{Height, Version, Weight};
use vecdb::Database;

use super::Vecs;
use crate::block_rolling::BlockRolling;

fn block_fullness(_: Height, weight: Weight) -> PartsPerMillion32 {
    PartsPerMillion32::from(weight.fullness())
}

impl Vecs {
    pub fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let weight = BlockRolling::new(
            "block_weight",
            version,
            &indexer.vecs().blocks.weight,
            PerBlockRolling::import(db, "block_weight", version, mappings, window_starts)?,
        );

        let fullness = LazyPercentVec::from_indexed_source(
            "block_fullness",
            version,
            &indexer.vecs().blocks.weight,
            block_fullness,
        );

        Ok(Self { weight, fullness })
    }
}
