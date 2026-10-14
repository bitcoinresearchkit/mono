use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, PerBlockRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use crate::block_rolling::BlockRolling;

impl Vecs {
    pub fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            size: BlockRolling::new(
                &indexer.vecs().blocks.total,
                PerBlockRolling::import(db, "block_size", version, mappings, window_starts)?,
            ),
        })
    }
}
