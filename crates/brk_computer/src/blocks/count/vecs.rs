use brk_traversable::Traversable;
use brk_types::StoredU64;

use crate::internal::{ConstantVecs, LazyPerBlockCumulativeRolling, Windows};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Expected number of blocks in the selected window at Bitcoin's target
    /// interval of ten minutes per block.
    pub target: Windows<ConstantVecs<StoredU64>>,
    /// Number of indexed blocks. The per-block value is one, the cumulative
    /// count is height plus one because genesis is included, and rolling sums
    /// count the blocks in the selected window.
    pub total: LazyPerBlockCumulativeRolling<StoredU64>,
}
