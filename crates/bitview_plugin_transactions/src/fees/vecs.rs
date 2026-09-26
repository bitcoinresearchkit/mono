use bitview_traversable::Traversable;
use brk_types::{FeeRate, Height, Sats, StoredBool, TxIndex};
use derive_more::{Deref, DerefMut};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use bitview_vecs::PerTxDistribution;

mod count;
mod cpfp_flags;

pub use count::CountVecs;
pub use cpfp_flags::CpfpFlags;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub count: CountVecs<M>,
    /// Coinbase output sum retained from the fee pass for mining rewards.
    #[traversable(hidden)]
    pub coinbase_value: M::Stored<EagerVec<PcoVec<Height, Sats>>>,
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
    /// Single Fee Linearization (SFL) independently to each same-block dependency
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
    pub cpfp_flags: CpfpFlags<M::Stored<EagerVec<PcoVec<TxIndex, StoredBool>>>>,
}
