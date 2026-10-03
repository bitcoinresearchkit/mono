use bitview_primitives::StoredU64;
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use vecdb::{Rw, StorageMode};

/// Transaction counts by detected structural pattern.
///
/// These are heuristic classifications of transactions, not protocol labels.
#[derive(Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Counts transactions heuristically classified as CoinJoin candidates:
    /// at least five inputs and outputs, neither count five times the other,
    /// sufficiently repeated input/output values, no recognized address reuse,
    /// and no detected `OP_RETURN` or inscription.
    pub coinjoin: PerBlockCumulativeRolling<StoredU64, M>,
    /// Counts transactions with at least five times as many inputs as outputs.
    pub consolidation: PerBlockCumulativeRolling<StoredU64, M>,
    /// Counts non-coinbase transactions with at least five times as many outputs
    /// as inputs.
    pub batch_payout: PerBlockCumulativeRolling<StoredU64, M>,
}

impl CountVecs {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PerBlockCumulativeRolling<StoredU64>> {
        [
            &mut self.coinjoin,
            &mut self.consolidation,
            &mut self.batch_payout,
        ]
        .into_iter()
    }
}
