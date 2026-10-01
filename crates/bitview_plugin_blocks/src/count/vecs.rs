use bitview_collections::Windows;
use bitview_traversable::Traversable;
use bitview_vecs::{ConstantVecs, LazyPerBlockCumulativeRolling};
use brk_types::StoredU64;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Expected number of blocks in a trailing window at Bitcoin's target
    /// interval of ten minutes per block.
    pub(crate) target: Windows<ConstantVecs<StoredU64>>,
    /// Number of indexed blocks. The per-block value is one, the cumulative
    /// count is height plus one because genesis is included, and rolling sums
    /// count the blocks in each supported trailing window.
    pub total: LazyPerBlockCumulativeRolling<StoredU64>,
}
