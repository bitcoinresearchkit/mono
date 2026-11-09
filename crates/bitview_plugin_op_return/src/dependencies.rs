use bitview_plugin_indexer::Indexer;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height, Sats, TxIndex};
use vecdb::PcoVec;

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub fee: &'a PcoVec<TxIndex, Sats>,
    pub spot_cents: &'a CachedSeries<Height, Cents>,
}
