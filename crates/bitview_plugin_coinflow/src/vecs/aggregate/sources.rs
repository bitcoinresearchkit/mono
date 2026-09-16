use bitview_cohort::UTXOAggregate;
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_types::{BoundedRatio, Cents, Height, Sats};
use vecdb::{Rw, StorageMode};

use super::super::{super::Horizons, Mobility};

#[derive(Traversable)]
pub struct AggregateSources<M: StorageMode = Rw> {
    pub supply: Mobility<UTXOAggregate<CachedSeries<Height, Sats, M>>>,
    /// Share of estimated mobile supply that is in loss: the sum of supply in
    /// loss multiplied by remaining-lifetime spending probability divided by
    /// the sum of total supply multiplied by that probability. Returns NaN
    /// when the weighted supply is zero.
    pub supply_in_loss_share: UTXOAggregate<CachedSeries<Height, BoundedRatio, M>>,
    /// For each supported forward horizon, the share of supply likely to move
    /// within that horizon that is in loss at the represented block. Each age
    /// range is weighted by one minus exp of the
    /// negative sum of its observed spending hazards times days across that
    /// horizon. Returns NaN when the weighted supply is zero.
    pub horizon: Horizons<UTXOAggregate<CachedSeries<Height, BoundedRatio, M>>>,
    /// Sum of creation-date USD value multiplied by remaining-lifetime spending
    /// probability across a set of UTXO age ranges. Creation-date value is each
    /// unspent output's BTC value multiplied by Bitcoin's spot price when it was
    /// created.
    pub cap: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Mobility-weighted mean creation price of supply estimated to move
    /// eventually: coinflow capitalization divided by estimated mobile supply
    /// in BTC. Returns zero when mobile supply is zero.
    pub price: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Creation price weighted by invested value and remaining-lifetime spending probability:
    /// sum(weight × creation price² × sats) / sum(weight × creation price × sats).
    /// Uses raw cost-basis moments; returns zero when weighted invested value is zero.
    pub capitalized_price: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    pub under_4m_price: CachedSeries<Height, Cents, M>,
    pub under_4m_capitalized_price: CachedSeries<Height, Cents, M>,
    pub under_6m_price: CachedSeries<Height, Cents, M>,
    pub under_6m_capitalized_price: CachedSeries<Height, Cents, M>,
    pub over_4m_price: CachedSeries<Height, Cents, M>,
    pub over_4m_capitalized_price: CachedSeries<Height, Cents, M>,
    pub over_6m_price: CachedSeries<Height, Cents, M>,
    pub over_6m_capitalized_price: CachedSeries<Height, Cents, M>,
}
