use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Difficulty, Hashrate};
use bitview_vecs::{CachedSeries, LazyPerBlock};
use brk_types::{Cents, Height, Sats};
use vecdb::PcoVec;

pub struct Dependencies<'a> {
    pub mappings: &'a MappingsVecs,
    pub difficulty_hashrate: &'a LazyPerBlock<Hashrate, Difficulty>,
    pub coinbase_value: &'a PcoVec<Height, Sats>,
    pub total_fees: &'a PcoVec<Height, Sats>,
    pub spot_cents: &'a CachedSeries<Height, Cents>,
}
