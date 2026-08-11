use brk_cohort::{TermId, UTXOAggregateId};
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, StoredF64};
use vecdb::{ColumnarVec, EagerVec, PcoVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct Sources<M: StorageMode = Rw> {
    pub awake_supply: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, TermId>>>,
    pub dormant_supply: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, TermId>>>,
    pub awake_cap: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, TermId>>>,
    pub awake_price: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, UTXOAggregateId>>>,
    pub supply_in_loss_share: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, StoredF64>, TermId>>>,
}
