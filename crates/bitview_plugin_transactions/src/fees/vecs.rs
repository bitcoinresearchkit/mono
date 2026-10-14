use bitview_primitives::{Boolean, Count};
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::{FeeRate, Height, Sats, TxIndex};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use crate::{distribution::PerTxDistribution, flagged::Classified};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Coinbase output sum retained from the fee pass for mining rewards.
    #[traversable(hidden)]
    pub coinbase_value: M::Stored<EagerVec<PcoVec<Height, Sats>>>,
    /// Transaction fee in satoshis: input value minus output value; coinbase is
    /// zero. The statistics count each of the block's non-coinbase transactions
    /// equally, zero fees included, as mempool.space's fee percentiles: sorted,
    /// min the first, max the last and percentile `p` at
    /// `sorted[max(0, floor(n * p) - 1)]`; zero for a block without them.
    /// Time-period indexes take the period's final block.
    pub fee: PerTxDistribution<Sats, M>,
    /// Canonical block fee total, accumulated in the monetary pass.
    #[traversable(hidden)]
    pub total: M::Stored<EagerVec<PcoVec<Height, Sats>>>,
    /// Fees of every block through this one: the denominator of fee chain shares.
    #[traversable(hidden)]
    pub total_cumulative: M::Stored<EagerVec<PcoVec<Height, Sats>>>,
    /// Raw transaction fee rate in sat/vB: fee divided by virtual size and
    /// rounded upward to the nearest 0.001 sat/vB. Coinbase and zero-fee
    /// transactions are zero.
    pub(crate) fee_rate: M::Stored<EagerVec<PcoVec<TxIndex, FeeRate>>>,
    /// Effective transaction fee rate in sat/vB after linearizing each same-block
    /// dependency component by ancestor set and chunking it so chunk feerates never
    /// increase. Every transaction in a chunk receives
    /// the chunk's combined fees divided by combined virtual size, rounded
    /// upward to the nearest 0.001 sat/vB. The statistics weight the block's
    /// non-coinbase transactions, zero rates included, by virtual size, with
    /// Bitcoin Core's `getblockstats` rank rule (the first rate whose cumulative
    /// vsize reaches the percentile); min is the lower of the 1st percentile and
    /// the lowest rate among the transactions overlapping the block's last 2% of
    /// vsize, max the higher of the 99th percentile and the highest among those
    /// overlapping its first 2% (mempool.space's heuristic); null for a block
    /// without them. Time-period indexes take the period's final block.
    pub effective_fee_rate: PerTxDistribution<FeeRate, M>,
    /// Child-pays-for-parent (CPFP) parents: transactions whose effective (chunk) fee rate is
    /// higher than their raw fee rate because same-block descendants raise the rate at which
    /// their chunk is evaluated.
    pub(crate) cpfp_parent: Classified<M>,
    /// Child-pays-for-parent (CPFP) children: transactions whose effective (chunk) fee rate is
    /// lower than their raw fee rate because their fee raises the rate at which a same-block
    /// chunk is evaluated.
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
