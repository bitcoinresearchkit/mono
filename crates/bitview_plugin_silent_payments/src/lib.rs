//! BIP352 silent payments for Bitview: every eligible transaction since Taproot's activation
//! (above a dust floor), with its tweak and its Taproot outputs' key prefixes, the data a wallet
//! scans with. Computed from the indexer's columns and Core's block files; no curve math is left
//! for a request to do.

mod compute;
mod dependencies;
mod import;
mod indexes;
mod scan;
#[cfg(test)]
mod tests;
mod tweak;

pub use dependencies::Dependencies;
pub use indexes::{TaprootPrefixIndex, TweakIndex};
pub use scan::DUST_FLOOR;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{Height, TaprootPrefix, TweakX, TxIndex, Version};
use vecdb::{AnyStoredVec, BytesVec, Database, PcoVec, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("silent_payments"), Version::new(3));
pub const ID: PluginId = STORAGE.id();

/// BIP352 silent payments: every eligible transaction since Taproot's activation with a Taproot
/// output of at least the dust floor, with its tweak and its Taproot outputs' key prefixes. Whether
/// an output is spent is the Outputs plugin's.
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
    /// Its tweak's first byte, 2 or 3 (the parity of its y): a column of its own, where it
    /// compresses to a bit.
    #[traversable(hidden)]
    pub tweak_parity: M::Stored<PcoVec<TweakIndex, u8>>,
    /// Its tweak's x coordinate.
    #[traversable(hidden)]
    pub tweak_x: M::Stored<BytesVec<TweakIndex, TweakX>>,
    /// Its first Taproot output's key prefix.
    #[traversable(hidden)]
    pub first_taproot_prefix_index: M::Stored<PcoVec<TweakIndex, TaprootPrefixIndex>>,
    /// Each Taproot output's key prefix, the transaction's outputs in order.
    #[traversable(hidden)]
    pub taproot_prefix: M::Stored<BytesVec<TaprootPrefixIndex, TaprootPrefix>>,
}

impl Vecs {
    fn stored_mut(&mut self) -> [&mut dyn AnyStoredVec; 6] {
        [
            &mut self.first_tweak_index,
            &mut self.tx_index,
            &mut self.tweak_parity,
            &mut self.tweak_x,
            &mut self.first_taproot_prefix_index,
            &mut self.taproot_prefix,
        ]
    }
}
