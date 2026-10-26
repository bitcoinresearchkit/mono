use bitview_primitives::{Count, Days, Difficulty, DifficultyEpoch, Hashrate, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, LazyPercentPerBlock, Resolutions};
use brk_types::Height;
use vecdb::LazyVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Mining difficulty encoded by the block header: Bitcoin's maximum target
    /// divided by the block's proof-of-work target.
    #[traversable(lead = "Difficulty: how hard it is to find a block, adjusted every 2,016 blocks to keep them ten minutes apart.")]
    pub value: Resolutions<Difficulty>,
    // The indexer's per-block difficulty: the height index of `value`, one series.
    /// Mining difficulty encoded by the block header: Bitcoin's maximum target
    /// divided by the block's proof-of-work target.
    #[traversable(rename = "value")]
    #[traversable(lead = "Difficulty: how hard it is to find a block, adjusted every 2,016 blocks to keep them ten minutes apart.")]
    pub(crate) block_height: LazyVec<Height, Difficulty, Difficulty>,
    /// Theoretical hash rate implied by difficulty at the ten-minute target:
    /// difficulty multiplied by 2^32 and divided by 600, in hashes per second.
    /// This is the rate expected to find one block every ten minutes at that
    /// difficulty, not an estimate from observed block production.
    #[traversable(lead = "Hash rate implied by difficulty: the computing power difficulty is set for.")]
    pub hashrate: LazyPerBlock<Hashrate, Difficulty>,
    /// Relative difficulty change versus 2,016 block heights earlier:
    /// represented-block difficulty divided by lookback difficulty, minus one.
    /// Positive values mean difficulty increased and negative values mean it
    /// decreased. Unavailable for the first 2,016 blocks.
    #[traversable(lead = "Difficulty change: how much difficulty moved over the last 2,016 blocks.")]
    pub(crate) adjustment: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Zero-based difficulty epoch: block height divided by 2,016, rounded down.
    #[traversable(lead = "Difficulty epoch: which 2,016-block period each block is in.")]
    pub(crate) epoch: LazyPerBlock<DifficultyEpoch>,
    /// Number of blocks from the represented height to the first block of the
    /// next difficulty epoch: 2,016 minus height modulo 2,016.
    #[traversable(lead = "Blocks until the next difficulty adjustment.")]
    pub(crate) blocks_to_retarget: LazyPerBlock<Count>,
    /// Nominal days to the next difficulty epoch, calculated as
    /// `blocks_to_retarget / 144`; this does not use observed mining pace.
    #[traversable(lead = "Days until the next difficulty adjustment, at ten minutes a block.")]
    pub(crate) days_to_retarget: LazyPerBlock<Days, Count>,
}
