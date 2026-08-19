use bitview_compute::{
    CachedWindowStartVec, LazyPerBlockRolling, LazyPercentVec, VBytesToWeight, Windows,
};
use bitview_plugin_indexer::Indexer;
use brk_types::{Height, PartsPerMillion32, Version, Weight};

use super::Vecs;
use crate::SizeVecs;

fn block_fullness(_: Height, weight: Weight) -> PartsPerMillion32 {
    PartsPerMillion32::from(weight.fullness())
}

pub trait Import {
    fn new(
        version: Version,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        size: &SizeVecs,
    ) -> Self;
}

impl Import for Vecs {
    fn new(
        version: Version,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        size: &SizeVecs,
    ) -> Self {
        let weight = LazyPerBlockRolling::from_full_parts::<VBytesToWeight>(
            "block_weight",
            version,
            &size.vbytes.cumulative,
            &size.vbytes.rolling,
            cached_starts,
            indexes,
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
