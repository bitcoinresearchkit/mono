use bitview_traversable::Traversable;
use brk_types::{Halving, StoredF32, StoredU32};

use bitview_compute::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Zero-based block-subsidy era, equal to block height divided by 210,000
    /// and rounded down. Era zero begins at genesis.
    pub epoch: LazyPerBlock<Halving>,
    /// Number of blocks from the represented height to the first block of the
    /// next subsidy era: 210,000 minus height modulo 210,000.
    pub blocks_to_halving: LazyPerBlock<StoredU32>,
    /// Nominal days to the next subsidy halving, calculated as
    /// `blocks_to_halving / 144`; this does not use observed mining pace.
    pub days_to_halving: LazyPerBlock<StoredF32, StoredU32>,
}
