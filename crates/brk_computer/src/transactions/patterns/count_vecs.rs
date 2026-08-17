use brk_traversable::Traversable;
use brk_types::StoredU64;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use super::PatternId;
use crate::internal::{ColumnarPerBlockCumulativeRolling, LazyColumnPerBlockCumulativeRolling};

/// Transaction counts by detected structural pattern.
///
/// These are heuristic classifications of transactions, not protocol labels.
#[derive(Deref, DerefMut, Traversable)]
pub struct CountVecs<M: StorageMode = Rw> {
    /// Counts transactions heuristically classified as CoinJoin candidates:
    /// at least five inputs and outputs, neither count five times the other,
    /// sufficiently repeated input/output values, no recognized address reuse,
    /// and no detected `OP_RETURN` or inscription.
    pub coinjoin: LazyColumnPerBlockCumulativeRolling<StoredU64, PatternId>,
    /// Counts transactions with at least five times as many inputs as outputs.
    pub consolidation: LazyColumnPerBlockCumulativeRolling<StoredU64, PatternId>,
    /// Counts non-coinbase transactions with at least five times as many outputs
    /// as inputs.
    pub batch_payout: LazyColumnPerBlockCumulativeRolling<StoredU64, PatternId>,
    #[deref]
    #[deref_mut]
    #[traversable(hidden)]
    pub source: ColumnarPerBlockCumulativeRolling<StoredU64, PatternId, (), M>,
}
