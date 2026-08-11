use brk_cohort::{TermId, UTXOAggregateId};
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredF64};
use vecdb::{ColumnarVec, EagerVec, PcoVec, Rw, StorageMode};

use super::super::{super::Horizons, Mobility};

#[derive(Traversable)]
pub struct AggregateSources<M: StorageMode = Rw> {
    pub supply: Mobility<M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, TermId>>>>,
    pub supply_in_loss_share:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, StoredF64>, UTXOAggregateId>>>,
    pub horizon:
        Horizons<M::Stored<EagerVec<ColumnarVec<PcoVec<Height, StoredF64>, UTXOAggregateId>>>>,
    pub cap: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, TermId>>>,
    pub price: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, UTXOAggregateId>>>,
}
