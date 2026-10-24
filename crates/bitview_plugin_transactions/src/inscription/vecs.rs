use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPercentCumulativeRolling, ValuePerBlockCumulativeRolling};
use vecdb::{Rw, StorageMode};

use bitview_plugin_indexer::FlagView;

use crate::flagged::TxCount;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Transactions where at least one Taproot script-path input contains the Ordinals envelope
    /// prefix `OP_0 OP_IF PUSH 'ord'` in its tapscript. Whether the transaction is one of them.
    pub flag: FlagView,
    /// Transactions where at least one Taproot script-path input contains the Ordinals envelope
    /// prefix `OP_0 OP_IF PUSH 'ord'` in its tapscript.
    #[traversable(flatten)]
    pub count: TxCount<M>,
    /// Sum of the full transaction fees for transactions whose
    /// Taproot scripts contain a detected Ordinals envelope. Each transaction
    /// contributes once, regardless of its number of inscriptions. Fees of
    /// separate commit transactions without a detected envelope are excluded.
    pub fees: ValuePerBlockCumulativeRolling<M>,
    /// Inscription transactions' fees divided by all transaction fees in the chain over the
    /// same cumulative or trailing window; undefined (null) when those fees are zero.
    #[traversable(wrap = "fees")]
    pub chain_share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}
