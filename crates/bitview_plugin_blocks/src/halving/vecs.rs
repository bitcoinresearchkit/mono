use bitview_primitives::{Count, Days, Halving};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Zero-based block-subsidy era, equal to block height divided by 210,000
    /// and rounded down. Era zero begins at genesis.
    pub epoch: LazyPerBlock<Halving>,
    /// Number of blocks from the represented height to the first block of the
    /// next subsidy era: 210,000 minus height modulo 210,000.
    pub blocks_to_halving: LazyPerBlock<Count>,
    /// Nominal days to the next subsidy halving, calculated as
    /// `blocks_to_halving / 144`; this does not use observed mining pace.
    pub days_to_halving: LazyPerBlock<Days, Count>,
}
