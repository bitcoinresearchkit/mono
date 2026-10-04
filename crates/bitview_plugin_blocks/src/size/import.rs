use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::WeightToVSize;
use bitview_vecs::{LazyPerBlockRolling, LazyWindowStartVec, PerBlockRolling};
use brk_error::Result;
use brk_types::{Height, VSize, Version, Weight};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::{Vecs, vecs::VBytes};
use crate::WeightVecs;

fn block_vbytes(_: Height, weight: Weight) -> VSize {
    VSize::from(weight)
}

impl Vecs {
    pub fn import(
        db: &Database,
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        weight: &WeightVecs,
    ) -> Result<Self> {
        let vbytes = VBytes {
            block: LazyVec::init(
                "block_vbytes",
                version,
                indexer.vecs().blocks.weight.read_only_boxed_clone(),
                block_vbytes,
            ),
            rolling: LazyPerBlockRolling::from_rolling::<WeightToVSize>(
                "block_vbytes",
                version,
                &weight.weight,
                window_starts,
                mappings,
            ),
        };

        Ok(Self {
            vbytes,
            size: PerBlockRolling::import(db, "block_size", version, mappings, window_starts)?,
        })
    }
}
