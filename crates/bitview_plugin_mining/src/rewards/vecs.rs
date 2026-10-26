use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPercentCumulativeRolling, ValuePerBlockCumulativeRolling, ValuePerBlockFull,
};
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Sum of the output values of the block's coinbase transaction. This is
    /// the miner reward actually assigned to coinbase outputs, not the total
    /// reward available.
    #[traversable(lead = "Miner revenue: what miners were paid, new coins and fees together.")]
    pub coinbase: ValuePerBlockCumulativeRolling<M>,
    /// Coinbase output value minus the block's total transaction fees, floored
    /// at zero. This is a derived subsidy component, not the scheduled
    /// consensus subsidy, and can be lower when the miner leaves some available
    /// reward unclaimed.
    #[traversable(lead = "Block subsidy: the new bitcoin miners claimed, besides fees.")]
    pub subsidy: ValuePerBlockCumulativeRolling<M>,
    /// Sum of input value minus output value across the block's non-coinbase
    /// transactions.
    #[traversable(lead = "Fees: what transactions paid miners.")]
    pub fees: ValuePerBlockFull<M>,
    /// Portion of the available block reward not assigned to coinbase outputs:
    /// scheduled subsidy plus transaction fees minus coinbase output value.
    #[traversable(lead = "Unclaimed rewards: what miners could have taken and didn't, gone for good.")]
    pub unclaimed: ValuePerBlockCumulativeRolling<M>,
    /// One minus the subsidy share: the claimed fees divided by coinbase output
    /// value. Cumulative variants use cumulative totals; rolling variants use
    /// totals within the trailing window.
    #[traversable(wrap = "fees", rename = "share")]
    #[traversable(lead = "Fees' share of what miners were paid.")]
    pub fee_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
    /// The derived subsidy component divided by coinbase output value.
    /// Cumulative variants use cumulative totals; rolling variants use totals
    /// within the trailing window.
    #[traversable(wrap = "subsidy", rename = "share")]
    #[traversable(lead = "The subsidy's share of what miners were paid.")]
    pub subsidy_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}
