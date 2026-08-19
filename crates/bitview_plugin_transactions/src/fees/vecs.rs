use bitview_traversable::Traversable;
use brk_types::{FeeRate, Sats, StoredBool, TxIndex};
use derive_more::{Deref, DerefMut};
use vecdb::{ColumnarVec, EagerVec, LazyColumnVec, PcoVec, ReadOnlyColumnarVec, Rw, StorageMode};

use bitview_compute::PerTxDistribution;

mod count;
mod cpfp_flags;
mod cpfp_role_id;

pub use count::CountVecs;
pub use cpfp_flags::CpfpFlags;
pub use cpfp_role_id::CpfpRoleId;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub count: CountVecs<M>,
    /// Sum of the transaction's referenced previous-output values, in
    /// satoshis. Coinbase uses `Sats::MAX` as a sentinel because it has no
    /// previous outputs to spend.
    pub input_value: M::Stored<EagerVec<PcoVec<TxIndex, Sats>>>,
    /// Sum of the transaction's output values, in satoshis.
    pub output_value: M::Stored<EagerVec<PcoVec<TxIndex, Sats>>>,
    /// Transaction fee in satoshis: input value minus output value; coinbase is
    /// zero. The transaction-index series includes zero-fee transactions.
    /// Distribution series count every included transaction equally and
    /// exclude coinbase and zero-fee transactions, either in the represented
    /// block or the six-block window ending there; time-period indexes take the
    /// value from the period's final block.
    pub fee: PerTxDistribution<Sats, M>,
    /// Raw transaction fee rate in sat/vB: fee divided by virtual size and
    /// rounded upward to the nearest 0.001 sat/vB. Coinbase and zero-fee
    /// transactions are zero.
    pub fee_rate: M::Stored<EagerVec<PcoVec<TxIndex, FeeRate>>>,
    /// Effective transaction fee rate in sat/vB after applying Bitcoin Core's
    /// Single Fee Linearization independently to each same-block dependency
    /// component. Every transaction in an ancestor-closed SFL chunk receives
    /// the chunk's combined fees divided by combined virtual size, rounded
    /// upward to the nearest 0.001 sat/vB. The transaction-index series
    /// includes zero effective rates. Distribution series exclude coinbase and
    /// zero effective rates and weight percentile ranks by transaction virtual
    /// size, either in the represented block or the six-block window ending
    /// there; time-period indexes take the value from the period's final block.
    pub effective_fee_rate: PerTxDistribution<FeeRate, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub cpfp_flags: CpfpFlags<
        LazyColumnVec<ReadOnlyColumnarVec<PcoVec<TxIndex, StoredBool>, CpfpRoleId>, CpfpRoleId>,
    >,
    #[traversable(hidden)]
    pub cpfp_flags_source:
        M::Stored<EagerVec<ColumnarVec<PcoVec<TxIndex, StoredBool>, CpfpRoleId>>>,
}
