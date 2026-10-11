use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{PerBlockCumulativeRolling, PercentPerBlock, ValuePerBlockCumulativeRolling};
use vecdb::{Rw, StorageMode};

use bitview_plugin_indexer::FlagView;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions where at least one Taproot script-path input contains the Ordinals envelope
    /// prefix `OP_0 OP_IF PUSH 'ord'` in its tapscript. Whether the transaction is one of them.
    pub flag: FlagView,
    /// Transactions where at least one Taproot script-path input contains the Ordinals envelope
    /// prefix `OP_0 OP_IF PUSH 'ord'` in its tapscript.
    pub count: PerBlockCumulativeRolling<Count, M>,
    /// Sum of the full transaction fees for transactions whose
    /// Taproot scripts contain a detected Ordinals envelope. Each transaction
    /// contributes once, regardless of its number of inscriptions. Fees of
    /// separate commit transactions without a detected envelope are excluded.
    pub fees: ValuePerBlockCumulativeRolling<M>,
    /// Fees of transactions containing a detected inscription divided by all
    /// transaction fees in the represented block. Zero when the block has no
    /// fees. Time-period indexes take the share from the period's final block.
    #[traversable(wrap = "fees", rename = "chain_share")]
    pub fee_share: PercentPerBlock<PartsPerMillion32, M>,
}
