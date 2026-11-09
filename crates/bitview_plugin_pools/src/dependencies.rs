use bitview_plugin_indexer::Indexer;
use bitview_vecs::LazyPreviousDeltaVec;
use brk_types::{Cents, Height, Sats};

pub struct Dependencies<'a> {
    pub indexer: &'a Indexer,
    pub coinbase_sats: &'a LazyPreviousDeltaVec<Height, Sats>,
    pub coinbase_cents: &'a LazyPreviousDeltaVec<Height, Cents>,
    pub fee_sats: &'a LazyPreviousDeltaVec<Height, Sats>,
    pub fee_cents: &'a LazyPreviousDeltaVec<Height, Cents>,
}
