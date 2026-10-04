use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{FixedRatioPerBlock, PerBlockCumulativeRolling};
use brk_types::Sats;
use vecdb::{Rw, StorageMode};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Counts transactions containing at least one Taproot script-path input
    /// whose tapscript contains the Ordinals envelope prefix
    /// `OP_0 OP_IF PUSH 'ord'`.
    pub count: PerBlockCumulativeRolling<Count, M>,
    /// Sum of the full transaction fees, in satoshis, for transactions whose
    /// Taproot scripts contain a detected Ordinals envelope. Each transaction
    /// contributes once, regardless of its number of inscriptions. Fees of
    /// separate commit transactions without a detected envelope are excluded.
    pub fees: PerBlockCumulativeRolling<Sats, M>,
    /// Fees of transactions containing a detected inscription divided by all
    /// transaction fees in the represented block. Zero when the block has no
    /// fees. Time-period indexes take the share from the period's final block.
    pub fee_share: FixedRatioPerBlock<PartsPerMillion32, M>,
}
