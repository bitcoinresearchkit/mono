#![allow(clippy::type_complexity)]

mod count;
mod dependencies;
mod features;
mod fees;
mod patterns;
mod policy;
mod sigops;
mod size;
mod versions;
mod volume;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

use count::Vecs as CountVecs;
pub use dependencies::Dependencies;
use features::Vecs as FeaturesVecs;
pub use fees::Vecs as FeesVecs;
use patterns::Vecs as PatternsVecs;
use policy::Vecs as PolicyVecs;
use sigops::Vecs as SigopsVecs;
use size::Vecs as SizeVecs;
use versions::Vecs as VersionsVecs;
use volume::Vecs as VolumeVecs;

pub const ID: PluginId = PluginId::new("transactions");
const DB_NAME: &str = ID.as_str();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub count: CountVecs<M>,
    pub features: FeaturesVecs<M>,
    pub size: SizeVecs<M>,
    pub fees: FeesVecs<M>,
    pub patterns: PatternsVecs<M>,
    /// BRK's deterministic, transaction-local approximation of default Bitcoin
    /// Core relay standardness, selected by mainnet block height rather than
    /// actual node adoption. It checks transaction version, weight and stripped
    /// size, script and witness forms, signature-operation limits, dust, and
    /// `OP_RETURN` policy. The approximation changes after height 863,500 and
    /// at heights 905,000 and 921,000. It does not reconstruct fee-floor,
    /// mempool/package topology, conflict or replacement policy, or
    /// node-specific settings. Coinbase transactions are classified false.
    pub policy: PolicyVecs<M>,
    pub sigops: SigopsVecs<M>,
    /// Counts every transaction, including coinbase, by its signed 32-bit
    /// Bitcoin transaction version.
    pub versions: VersionsVecs<M>,
    pub volume: VolumeVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
