use bitview_traversable::Traversable;
use vecdb::{AnyStoredVec, Rw, StorageMode};

use crate::flagged::Classified;

/// Transactions by detected structural pattern: heuristic classifications, not protocol labels.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions heuristically classified as CoinJoin candidates: at least five inputs and
    /// outputs, neither count five times the other, sufficiently repeated input/output values, no
    /// recognized address reuse, and no detected `OP_RETURN` or inscription.
    #[traversable(lead = "CoinJoins: transactions that look like many people mixing their coins together.")]
    pub coinjoin: Classified<M>,
    /// Transactions with at least five times as many inputs as outputs.
    #[traversable(lead = "Consolidations: transactions gathering many inputs into few outputs.")]
    pub consolidation: Classified<M>,
    /// Non-coinbase transactions with at least five times as many outputs as inputs.
    #[traversable(lead = "Batch payouts: transactions paying many outputs from few inputs.")]
    pub batch_payout: Classified<M>,
}

impl Vecs {
    /// Every pattern's stored flags, then counts.
    pub(super) fn stored_mut(
        &mut self,
    ) -> ([&mut dyn AnyStoredVec; 3], [&mut dyn AnyStoredVec; 3]) {
        let (coinjoin_flag, coinjoin_count) = self.coinjoin.stored_mut();
        let (consolidation_flag, consolidation_count) = self.consolidation.stored_mut();
        let (batch_payout_flag, batch_payout_count) = self.batch_payout.stored_mut();
        (
            [coinjoin_flag, consolidation_flag, batch_payout_flag],
            [coinjoin_count, consolidation_count, batch_payout_count],
        )
    }
}
