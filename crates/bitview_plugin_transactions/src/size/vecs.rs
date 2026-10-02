use bitview_traversable::Traversable;
use brk_types::{VSize, Weight};
use vecdb::{Rw, StorageMode};

use bitview_vecs::{LazyPerTxDistributionTransformed, TxDerivedDistribution};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transaction virtual size in vbytes, calculated as BIP-141 weight divided
    /// by four and rounded up. The transaction-index series gives each
    /// transaction's value. Distribution series count every transaction
    /// equally and include coinbase, either in the represented block or the
    /// six-block window ending there; time-period indexes take the value from
    /// the period's final block.
    pub vsize: LazyPerTxDistributionTransformed<VSize, Weight, Weight>,
    /// BIP-141 transaction weight in weight units: non-witness bytes count as
    /// four weight units and witness bytes count as one. The transaction-index
    /// series gives each transaction's value. Distribution series count every
    /// transaction equally and include coinbase, either in the represented
    /// block or the six-block window ending there; time-period indexes take the
    /// value from the period's final block.
    pub weight: TxDerivedDistribution<Weight, M>,
}
