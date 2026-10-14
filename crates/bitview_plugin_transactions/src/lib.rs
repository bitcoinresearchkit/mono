#![allow(clippy::type_complexity)]

mod compute;
mod count;
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
mod sigops;
mod versions;
mod vsize;

pub use dependencies::Dependencies;
pub use fees::Vecs as FeesVecs;
pub use has::HasTransactions;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerSecondWindows, ValuePerBlockCumulativeRolling};
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use count::Vecs as CountVecs;
use features::Vecs as FeaturesVecs;
use inscription::Vecs as InscriptionVecs;
use patterns::Vecs as PatternsVecs;
use policy::Vecs as PolicyVecs;
use sigops::Vecs as SigopsVecs;
use versions::Vecs as VersionsVecs;
use vsize::Vecs as VSizeVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("transactions"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    pub count: CountVecs<M>,
    features: FeaturesVecs<M>,
    /// Transaction virtual size in vbytes: BIP-141 weight divided by four, rounded up. The
    /// statistics count each of the block's transactions equally, coinbase included, at
    /// mempool.space's ranks (min the first, max the last, percentile `p` at
    /// `sorted[max(0, floor(n * p) - 1)]`); time-period indexes take the period's final block.
    vsize: VSizeVecs<M>,
    pub fees: FeesVecs<M>,
    inscription: InscriptionVecs<M>,
    patterns: PatternsVecs<M>,
    #[traversable(flatten)]
    policy: PolicyVecs<M>,
    sigop_cost: SigopsVecs<M>,
    /// Counts every transaction, including coinbase, by its signed 32-bit
    /// Bitcoin transaction version.
    versions: VersionsVecs<M>,
    /// Sum of the input values of non-coinbase transactions. This equals their
    /// total output value plus transaction fees and is not adjusted to estimate
    /// economic payment volume.
    pub volume: ValuePerBlockCumulativeRolling<M>,
    /// Transaction rate, including coinbase transactions.
    per_second: LazyPerSecondWindows,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
