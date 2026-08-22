use bitview_cohort::{TermId, UTXOAggregateId};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredF64};
use vecdb::{ColumnarVec, EagerVec, PcoVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Sources<M: StorageMode = Rw> {
    /// Sum of supply multiplied by wakefulness across a set of UTXO age ranges.
    /// Each age-range contribution is rounded down to whole satoshis.
    pub awake_supply: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, TermId>>>,
    /// Sum of supply multiplied by one minus wakefulness across a set of UTXO
    /// age ranges. Each age-range contribution is rounded down to whole
    /// satoshis.
    pub dormant_supply: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, TermId>>>,
    /// Sum of creation-date USD value multiplied by wakefulness across a set of
    /// UTXO age ranges. Creation-date value is each unspent output's BTC value
    /// multiplied by Bitcoin's spot price when it was created.
    pub awake_cap: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, TermId>>>,
    /// Wakefulness-weighted mean creation price: awake capitalization divided
    /// by awake supply in BTC. Returns zero when awake supply is zero.
    pub awake_price: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, UTXOAggregateId>>>,
    /// Share of awake supply that is in loss: the sum of supply in loss
    /// multiplied by wakefulness divided by the sum of total supply multiplied
    /// by wakefulness. Returns NaN when the weighted supply is zero.
    pub supply_in_loss_share: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, StoredF64>, TermId>>>,
}
