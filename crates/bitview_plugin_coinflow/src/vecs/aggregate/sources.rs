use bitview_cohort::{TermId, UTXOAggregateId};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredF64};
use vecdb::{ColumnarVec, EagerVec, PcoVec, Rw, StorageMode};

use super::super::{super::Horizons, Mobility};

#[derive(Traversable)]
pub struct AggregateSources<M: StorageMode = Rw> {
    pub supply: Mobility<M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, TermId>>>>,
    /// Share of estimated mobile supply that is in loss: the sum of supply in
    /// loss multiplied by remaining-lifetime spending probability divided by
    /// the sum of total supply multiplied by that probability. Returns NaN
    /// when the weighted supply is zero.
    pub supply_in_loss_share:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, StoredF64>, UTXOAggregateId>>>,
    /// Share of supply likely to move within the named forward horizon that is
    /// currently in loss. Each age range is weighted by one minus exp of the
    /// negative sum of its observed spending hazards times days across that
    /// horizon. Returns NaN when the weighted supply is zero.
    pub horizon:
        Horizons<M::Stored<EagerVec<ColumnarVec<PcoVec<Height, StoredF64>, UTXOAggregateId>>>>,
    /// Sum of realized capitalization multiplied by remaining-lifetime
    /// spending probability across the selected UTXO age ranges.
    pub cap: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, TermId>>>,
    /// Coinflow capitalization divided by estimated mobile supply in BTC.
    /// Returns zero when mobile supply is zero.
    pub price: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, UTXOAggregateId>>>,
}
