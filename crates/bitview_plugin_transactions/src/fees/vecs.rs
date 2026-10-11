use bitview_primitives::{Boolean, Count};
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlockCumulativeRolling, PerTxDistribution};
use brk_types::{FeeRate, Height, Sats, TxIndex};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use crate::flagged::Classified;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
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
    /// Canonical block fee total, accumulated in the monetary pass.
    #[traversable(hidden)]
    pub total: M::Stored<EagerVec<PcoVec<Height, Sats>>>,
    /// Raw transaction fee rate in sat/vB: fee divided by virtual size and
    /// rounded upward to the nearest 0.001 sat/vB. Coinbase and zero-fee
    /// transactions are zero.
    pub(crate) fee_rate: M::Stored<EagerVec<PcoVec<TxIndex, FeeRate>>>,
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
    /// Child-pays-for-parent (CPFP) parents: transactions whose Single Fee Linearization (SFL)
    /// effective fee rate is higher than their raw fee rate because same-block descendants raise
    /// the rate at which their SFL chunk is evaluated.
    pub(crate) cpfp_parent: Classified<M>,
    /// Child-pays-for-parent (CPFP) children: transactions whose Single Fee Linearization (SFL)
    /// effective fee rate is lower than their raw fee rate because their fee raises the rate at
    /// which a same-block ancestor-closed SFL chunk is evaluated.
    pub(crate) cpfp_child: Classified<M>,
}

impl Vecs {
    fn cpfp_roles_mut(&mut self) -> [&mut Classified<Rw>; 2] {
        [&mut self.cpfp_parent, &mut self.cpfp_child]
    }

    pub(super) fn cpfp_flags_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut EagerVec<PcoVec<TxIndex, Boolean>>> {
        self.cpfp_roles_mut().into_iter().map(|role| &mut role.flag)
    }

    pub(super) fn cpfp_counts_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut PerBlockCumulativeRolling<Count>> {
        self.cpfp_roles_mut()
            .into_iter()
            .map(|role| &mut role.count)
    }
}
