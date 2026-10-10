use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::WeightToVSize;
use bitview_vecs::{LazyPerBlockRolling, LazyWindowStartVec, PerBlockRolling};
use brk_error::Result;
use brk_types::{Height, VSize, Version, Weight};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::{Vecs, vecs::VirtualSize};
use crate::{WeightVecs, block_rolling::BlockRolling};

fn block_vsize(_: Height, weight: Weight) -> VSize {
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
        let vsize = VirtualSize {
            block: LazyVec::init(
                "block_vsize",
                version,
                indexer.vecs().blocks.weight.read_only_boxed_clone(),
                block_vsize,
            ),
            rolling: LazyPerBlockRolling::from_rolling::<WeightToVSize>(
                "block_vsize",
                version,
                &weight.weight,
                window_starts,
                mappings,
            ),
        };

        Ok(Self {
            vsize,
            size: BlockRolling::new(
                "block_size",
                version,
                &indexer.vecs().blocks.total,
                PerBlockRolling::import(db, "block_size", version, mappings, window_starts)?,
            ),
        })
    }
}
