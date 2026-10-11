use bitview_primitives::{Boolean, Count};
use bitview_traversable::Traversable;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::TxIndex;
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use crate::flagged::Classified;

/// Transactions by detected structural pattern: heuristic classifications, not protocol labels.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions heuristically classified as CoinJoin candidates: at least five inputs and
    /// outputs, neither count five times the other, sufficiently repeated input/output values, no
    /// recognized address reuse, and no detected `OP_RETURN` or inscription.
    pub coinjoin: Classified<M>,
    /// Transactions with at least five times as many inputs as outputs.
    pub consolidation: Classified<M>,
    /// Non-coinbase transactions with at least five times as many outputs as inputs.
    pub batch_payout: Classified<M>,
}

impl Vecs {
    fn patterns_mut(&mut self) -> [&mut Classified<Rw>; 3] {
        [
            &mut self.coinjoin,
            &mut self.consolidation,
            &mut self.batch_payout,
        ]
    }

    pub(super) fn flags_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut EagerVec<PcoVec<TxIndex, Boolean>>> {
        self.patterns_mut()
            .into_iter()
            .map(|pattern| &mut pattern.flag)
    }

    pub(super) fn counts_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut PerBlockCumulativeRolling<Count>> {
        self.patterns_mut()
            .into_iter()
            .map(|pattern| &mut pattern.count)
    }
}
