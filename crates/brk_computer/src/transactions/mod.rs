pub mod count;
pub mod features;
pub mod fees;
pub mod patterns;
pub mod policy;
pub mod sigops;
pub mod size;
pub mod versions;
pub mod volume;

mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use brk_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

pub use count::Vecs as CountVecs;
pub use features::Vecs as FeaturesVecs;
pub use fees::Vecs as FeesVecs;
pub use patterns::Vecs as PatternsVecs;
pub use policy::Vecs as PolicyVecs;
pub use sigops::Vecs as SigopsVecs;
pub use size::Vecs as SizeVecs;
pub use versions::Vecs as VersionsVecs;
pub use volume::Vecs as VolumeVecs;

pub const DB_NAME: &str = "transactions";

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,

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
        PluginId::new(DB_NAME)
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
