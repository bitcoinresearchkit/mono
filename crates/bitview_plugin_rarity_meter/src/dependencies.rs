use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height};

/// Spot price, read here. The three plugins are borrowed only to force them to finish first:
/// the reference prices are read-only handles bound at import, and only see this block once
/// their producers have computed it.
pub struct Dependencies<'a> {
    pub holders: &'a HoldersVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
    pub spot_cents: &'a CachedSeries<Height, Cents>,
}
