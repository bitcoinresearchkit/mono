use bitview_primitives::{Count, Days, HalvingEpoch};
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Zero-based halving epoch (block-subsidy era): block height divided by 210,000, rounded
    /// down; epoch zero begins at genesis.
    #[traversable(lead = "Halving epoch: which subsidy era each block is in, one every 210,000 blocks.")]
    pub epoch: LazyPerBlock<HalvingEpoch>,
    /// Number of blocks from the represented height to the first block of the
    /// next subsidy era: 210,000 minus height modulo 210,000.
    #[traversable(lead = "Blocks until the next halving.")]
    pub blocks_to_halving: LazyPerBlock<Count>,
    /// Nominal days to the next subsidy halving, calculated as
    /// `blocks_to_halving / 144`; this does not use observed mining pace.
    #[traversable(lead = "Days until the next halving, at ten minutes a block.")]
    pub days_to_halving: LazyPerBlock<Days, Count>,
}
