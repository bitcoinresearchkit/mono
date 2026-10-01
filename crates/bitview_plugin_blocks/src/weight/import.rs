use bitview_collections::Windows;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::VBytesToWeight;
use bitview_vecs::{LazyPerBlockRolling, LazyPercentVec, LazyWindowStartVec};
use brk_types::{Height, PartsPerMillion32, Version, Weight};

use super::Vecs;
use crate::UtxosVecs;

fn block_fullness(_: Height, weight: Weight) -> PartsPerMillion32 {
    PartsPerMillion32::from(weight.fullness())
}

impl Vecs {
    pub fn new(
        version: Version,
        indexer: &Indexer,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        size: &UtxosVecs,
    ) -> Self {
        let weight = LazyPerBlockRolling::from_full_parts::<VBytesToWeight>(
            "block_weight",
            version,
            &size.vbytes.cumulative,
            &size.vbytes.rolling,
            window_starts,
            mappings,
        );

        let fullness = LazyPercentVec::from_indexed_source(
            "block_fullness",
            version,
            &indexer.vecs().blocks.weight,
            block_fullness,
        );

        Self { weight, fullness }
    }
}
