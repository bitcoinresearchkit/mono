use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::StoredU64;
use bitview_vecs::{LazyWindowStartVec, PerBlockFull, PerBlockRolling};
use brk_error::Result;
use brk_types::{Height, Version, Weight};
use vecdb::Database;

use super::Vecs;

fn block_vbytes(_: Height, weight: Weight) -> StoredU64 {
    StoredU64::from(weight.to_vbytes_floor())
}

impl Vecs {
    pub fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            vbytes: PerBlockFull::import(
                db,
                "block_vbytes",
                version,
                &indexer.vecs().blocks.weight,
                block_vbytes,
                mappings,
                window_starts,
            )?,
            size: PerBlockRolling::import(db, "block_size", version, mappings, window_starts)?,
        })
    }
}
