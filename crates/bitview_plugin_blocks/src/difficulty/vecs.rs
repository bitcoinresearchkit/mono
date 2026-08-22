use bitview_traversable::Traversable;
use brk_types::{Epoch, PartsPerMillionSigned32, StoredF32, StoredF64, StoredU32};

use bitview_compute::{LazyPerBlock, LazyPercentPerBlock, Resolutions};

#[derive(Clone, Traversable)]
pub struct Vecs {
    /// Mining difficulty encoded by the block header, calculated as Bitcoin's
    /// maximum target divided by this block's proof-of-work target. A larger
    /// value means a lower valid-hash target and therefore more expected hashing
    /// work per block.
    pub value: Resolutions<StoredF64>,
    /// Theoretical hash rate implied by difficulty at the ten-minute target:
    /// difficulty multiplied by 2^32 and divided by 600, in hashes per second.
    /// This is the rate expected to find one block every ten minutes at that
    /// difficulty, not an estimate from observed block production.
    pub hashrate: LazyPerBlock<StoredF64>,
    /// Relative difficulty change versus 2,016 block heights earlier:
    /// represented-block difficulty divided by lookback difficulty, minus one.
    /// Positive values mean difficulty increased and negative values mean it
    /// decreased. Unavailable for the first 2,016 blocks.
    pub adjustment: LazyPercentPerBlock<PartsPerMillionSigned32>,
    /// Zero-based difficulty epoch number, equal to block height divided by
    /// 2,016 and rounded down.
    pub epoch: LazyPerBlock<Epoch>,
    /// Number of blocks from the represented height to the first block of the
    /// next difficulty epoch: 2,016 minus height modulo 2,016.
    pub blocks_to_retarget: LazyPerBlock<StoredU32>,
    /// Nominal days to the next difficulty epoch, calculated as
    /// `blocks_to_retarget / 144`; this does not use observed mining pace.
    pub days_to_retarget: LazyPerBlock<StoredF32, StoredU32>,
}
