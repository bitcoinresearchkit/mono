//! BIP352 silent payments for Bitview: every eligible transaction since Taproot's activation,
//! with its tweak, the data a wallet scans with. Computed from the indexer's columns and Core's
//! block files; no curve math is left for a request to do.

mod compute;
mod dependencies;
mod import;
mod scan;
#[cfg(test)]
mod tests;
mod tweak;
mod tweak_index;

pub use dependencies::Dependencies;
pub use tweak_index::TweakIndex;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{Height, Tweak, TxIndex, Version};
use vecdb::{AnyStoredVec, BytesVec, Database, PcoVec, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("silent_payments"), Version::ONE);
pub const ID: PluginId = STORAGE.id();

/// BIP352 silent payments: every eligible transaction since Taproot's activation, with its tweak.
/// Hidden from the series tree: tweaks are scanning data, not series to chart.
#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    /// Each block's first eligible transaction.
    #[traversable(hidden)]
    pub first_tweak_index: M::Stored<PcoVec<Height, TweakIndex>>,
    /// The eligible transaction.
    #[traversable(hidden)]
    pub tx_index: M::Stored<PcoVec<TweakIndex, TxIndex>>,
    /// Its tweak.
    #[traversable(hidden)]
    pub tweak: M::Stored<BytesVec<TweakIndex, Tweak>>,
}

impl Vecs {
    fn stored_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [&mut self.first_tweak_index, &mut self.tx_index, &mut self.tweak]
    }
}
