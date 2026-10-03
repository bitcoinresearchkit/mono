#![allow(clippy::type_complexity)]

mod count;
mod dependencies;
mod features;
mod fees;
mod has;
mod inscription;
mod patterns;
mod policy;
mod sigops;
mod size;
mod versions;
mod volume;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
use features::Vecs as FeaturesVecs;
pub use fees::Vecs as FeesVecs;
pub use has::HasTransactions;
use inscription::Vecs as InscriptionVecs;
use patterns::Vecs as PatternsVecs;
use policy::Vecs as PolicyVecs;
use sigops::Vecs as SigopsVecs;
use size::Vecs as UtxosVecs;
use versions::Vecs as VersionsVecs;
use volume::Vecs as VolumeVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("transactions"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    pub count: CountVecs<M>,
    features: FeaturesVecs<M>,
    size: UtxosVecs<M>,
    pub fees: FeesVecs<M>,
    inscription: InscriptionVecs<M>,
    pub patterns: PatternsVecs<M>,
    /// BRK's transaction-local approximation of default Bitcoin Core relay
    /// standardness at the represented block height. It checks version,
    /// size and weight, scripts and witnesses, signature-operation limits,
    /// dust, and `OP_RETURN` policy. It does not model node adoption, fee
    /// floors, mempool or package topology, conflicts or replacement, or
    /// node-specific settings. Modeled rules change after height 863,500 and at
    /// heights 905,000 and 921,000; coinbase is classified nonstandard.
    policy: PolicyVecs<M>,
    sigops: SigopsVecs<M>,
    /// Counts every transaction, including coinbase, by its signed 32-bit
    /// Bitcoin transaction version.
    versions: VersionsVecs<M>,
    pub volume: VolumeVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
