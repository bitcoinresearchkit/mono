use bitview_cohort::{ByTerm, UTXOAggregate};
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_types::{BoundedRatio, Cents, Height, Sats};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Sources<M: StorageMode = Rw> {
    /// Sum of supply multiplied by wakefulness across a set of UTXO age ranges.
    /// Each age-range contribution is rounded down to whole satoshis.
    pub awake_supply: UTXOAggregate<CachedSeries<Height, Sats, M>>,
    /// Sum of supply multiplied by one minus wakefulness across a set of UTXO
    /// age ranges. Each age-range contribution is rounded down to whole
    /// satoshis.
    pub dormant_supply: UTXOAggregate<CachedSeries<Height, Sats, M>>,
    /// Sum of creation-date USD value multiplied by wakefulness across a set of
    /// UTXO age ranges. Creation-date value is each unspent output's BTC value
    /// multiplied by Bitcoin's spot price when it was created.
    pub awake_cap: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Wakefulness-weighted mean creation price: awake capitalization divided
    /// by awake supply in BTC. Returns zero when awake supply is zero.
    pub awake_price: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Creation price weighted by invested value and wakefulness:
    /// sum(weight × creation price² × sats) / sum(weight × creation price × sats).
    /// Uses raw cost-basis moments; returns zero when weighted invested value is zero.
    pub awake_capitalized_price: UTXOAggregate<CachedSeries<Height, Cents, M>>,
    /// Share of awake supply that is in loss: the sum of supply in loss
    /// multiplied by wakefulness divided by the sum of total supply multiplied
    /// by wakefulness. Returns NaN when the weighted supply is zero.
    pub supply_in_loss_share: ByTerm<CachedSeries<Height, BoundedRatio, M>>,
    pub under_4m_awake_price: CachedSeries<Height, Cents, M>,
    pub under_4m_awake_capitalized_price: CachedSeries<Height, Cents, M>,
    pub under_6m_awake_price: CachedSeries<Height, Cents, M>,
    pub under_6m_awake_capitalized_price: CachedSeries<Height, Cents, M>,
    pub over_4m_awake_price: CachedSeries<Height, Cents, M>,
    pub over_4m_awake_capitalized_price: CachedSeries<Height, Cents, M>,
    pub over_6m_awake_price: CachedSeries<Height, Cents, M>,
    pub over_6m_awake_capitalized_price: CachedSeries<Height, Cents, M>,
}
