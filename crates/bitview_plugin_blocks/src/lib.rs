mod block_rolling;
mod compute;
mod count;
mod counts;
mod dependencies;
mod difficulty;
mod fullness;
mod halving;
mod has;
mod import;
mod interval;

pub use dependencies::Dependencies;
pub use difficulty::Vecs as DifficultyVecs;
pub use has::HasBlocks;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{Bytes32, Count, Work};
use bitview_traversable::Traversable;
use bitview_vecs::{CumulativeRolling, LazyCumulativeRolling, PerBlockCumulativeRolling};
use brk_types::{SigOps, Version, Weight32};
use vecdb::{Database, Rw, StorageMode};

use block_rolling::BlockRolling;
use counts::CoinbaseLocktime;
use fullness::Fullness;
use halving::Vecs as HalvingVecs;
use interval::Vecs as IntervalVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("blocks"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Number of indexed blocks: the cumulative count is height plus one because
    /// genesis is included, and rolling sums count the blocks in each supported
    /// trailing window.
    #[traversable(lead = "Blocks: how many were mined.")]
    count: LazyCumulativeRolling<Count>,
    interval: IntervalVecs<M>,
    /// Serialized block size in bytes, including witness data: its 80-byte header,
    /// transaction-count CompactSize, and every serialized transaction.
    #[traversable(lead = "Block size: the bytes each block takes, witness data included.")]
    pub size: BlockRolling<Bytes32, M>,
    /// BIP-141 block weight in weight units: non-witness bytes count as four
    /// weight units and witness bytes count as one.
    #[traversable(lead = "Block weight: each block measured against the limit of 4 million weight units.")]
    weight: BlockRolling<Weight32, M>,
    /// Block total of BIP-141 signature-operation cost, including coinbase:
    /// legacy scriptPubKey, scriptSig, and P2SH redeem-script sigops cost four
    /// units; P2WPKH and P2WSH sigops cost one. This is a static count, not the
    /// number of signatures executed. Tapscript sigops are excluded because
    /// BIP-342 uses a separate per-input budget. The post-SegWit block limit is
    /// 80,000 cost units.
    #[traversable(lead = "Signature operations: how much signature checking blocks ask of nodes.")]
    sigop_cost: PerBlockCumulativeRolling<SigOps, M>,
    fullness: Fullness,
    pub difficulty: DifficultyVecs,
    halving: HalvingVecs,
    /// Blocks holding only their coinbase transaction.
    #[traversable(lead = "Empty blocks: ones with nothing but the coinbase.")]
    empty_count: CumulativeRolling<Count, M>,
    /// Proof of work: each block's difficulty times 2^48 / 0xFFFF, the hashes expected to find
    /// it. Its running total is the chain work, a floating-point sum rather than Bitcoin Core's
    /// exact integer one.
    #[traversable(lead = "Proof of work: the hashes expected to find the blocks.")]
    work: PerBlockCumulativeRolling<Work, M>,
    /// Blocks with bit 4 set in a BIP-9 version, the bit BIP-110 signals with (and BIP-91 did in
    /// 2017): the top three version bits are 001 and bit 4 is 1.
    #[traversable(
        wrap = "version_bits",
        rename = "bit4",
        lead = "Blocks signaling bit 4, as BIP-110 does."
    )]
    version_bit_4: CumulativeRolling<Count, M>,
    coinbase_locktime: CoinbaseLocktime<M>,
}
