#![allow(clippy::type_complexity)]

mod compute;
mod dependencies;
mod distribution;
mod features;
mod fees;
mod flagged;
mod has;
mod import;
mod inscription;
mod patterns;
mod policy;
mod resume;
mod versions;
mod vsize;

pub use dependencies::Dependencies;
pub use fees::Vecs as FeesVecs;
pub use has::HasTransactions;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Count16;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerSecondWindows, PerBlockCumulativeRolling, PerBlockFullFromCumulative,
    ValuePerBlockCumulativeRolling,
};
use brk_types::{SigOps, Version};
use vecdb::{Database, Rw, StorageMode};

use features::Vecs as FeaturesVecs;
use flagged::Classified;
use inscription::Vecs as InscriptionVecs;
use patterns::Vecs as PatternsVecs;
use versions::Vecs as VersionsVecs;
use vsize::Vecs as VSizeVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("transactions"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Number of transactions, including the coinbase transaction.
    pub count: PerBlockFullFromCumulative<Count16, M>,
    features: FeaturesVecs<M>,
    /// Transaction virtual size in vbytes: BIP-141 weight divided by four, rounded up. The
    /// statistics count each of the block's transactions equally, coinbase included, at
    /// mempool.space's ranks (min the first, max the last, percentile `p` at
    /// `sorted[max(0, floor(n * p) - 1)]`); time-period indexes take the period's final block.
    vsize: VSizeVecs<M>,
    pub fees: FeesVecs<M>,
    inscription: InscriptionVecs<M>,
    patterns: PatternsVecs<M>,
    /// Transactions nonstandard under BRK's transaction-local approximation of
    /// default Bitcoin Core relay standardness at the represented block height.
    /// It checks version, size and weight, scripts and witnesses,
    /// signature-operation limits, dust, and `OP_RETURN` policy. It does not
    /// model node adoption, fee floors, mempool or package topology, conflicts or
    /// replacement, or node-specific settings. Modeled rules change after height
    /// 863,500 and at heights 892,500 and 921,000; coinbase transactions are not
    /// evaluated (never nonstandard).
    nonstandard: Classified<M>,
    /// Block total of BIP-141 signature-operation cost, including coinbase:
    /// legacy scriptPubKey, scriptSig, and P2SH redeem-script sigops cost four
    /// units; P2WPKH and P2WSH sigops cost one. This is a static count, not the
    /// number of signatures executed. Tapscript sigops are excluded because
    /// BIP-342 uses a separate per-input budget. The post-SegWit block limit is
    /// 80,000 cost units.
    sigop_cost: PerBlockCumulativeRolling<SigOps, M>,
    /// Counts every transaction, including coinbase, by its signed 32-bit
    /// Bitcoin transaction version.
    versions: VersionsVecs<M>,
    /// Sum of the input values of non-coinbase transactions. This equals their
    /// total output value plus transaction fees and is not adjusted to estimate
    /// economic payment volume.
    pub transfer_volume: ValuePerBlockCumulativeRolling<M>,
    /// Transaction rate, including coinbase transactions.
    per_second: LazyPerSecondWindows,
}
