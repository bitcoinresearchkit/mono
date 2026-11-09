use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_vecs::CachedSeries;
use brk_types::{Cents, Height};

/// Spot price, read here. The three plugins are borrowed only so the composition runs the meter
/// after them: the reference prices and ratios are read-only handles bound at import (the bands
/// are lazy views over them), and `Component::compute` checks in dev that they hold this
/// update's rows.
pub struct Dependencies<'a> {
    pub holders: &'a HoldersVecs,
    pub cointime: &'a CointimeVecs,
    pub coinflow: &'a CoinflowVecs,
    pub spot_cents: &'a CachedSeries<Height, Cents>,
}
