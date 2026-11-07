//! BIP352 silent payments for Bitview: every eligible transaction since Taproot's activation
//! (above a dust floor) with its tweak, and each block's filters of new Taproot outputs and of
//! spent ones: what a light wallet scans with. Computed from the indexer's columns and Core's
//! block files; no curve math is left for a request to do.

mod compute;
mod dependencies;
mod filters;
mod has;
mod import;
mod indexes;
mod scan;
#[cfg(test)]
mod tests;
mod tweak;

pub use dependencies::Dependencies;
pub use filters::spent_outputs;
pub use has::HasSilentPayments;
pub use indexes::{FilterByteIndex, TweakIndex};
pub use scan::{DUST_FLOOR, TAPROOT_ACTIVATION};

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{Height, TweakX, TxIndex, Version};
use vecdb::{AnyStoredVec, BytesVec, Database, PcoVec, Rw, StorageMode};

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("silent_payments"), Version::new(6));
pub const ID: PluginId = STORAGE.id();

/// BIP352 silent payments: every eligible transaction since Taproot's activation with a Taproot
/// output of at least the dust floor, with its tweak, and each block's two filters. Outputs,
/// values and spends are read from the indexer and the Outputs plugin.
/// Hidden from the series tree: tweaks are scanning data, not series to chart.
#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    /// Each block's end among the eligible transactions: how many it and the blocks before hold.
    /// Ends, not firsts: a block's rows are read from its height and the one before alone, never
    /// from a column's length, which a block being appended or rolled back may have moved.
    #[traversable(hidden)]
    pub end_tweak_index: M::Stored<PcoVec<Height, TweakIndex>>,
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
    /// Each block's end among the new Taproot outputs' filter bytes.
    #[traversable(hidden)]
    pub end_new_outputs_filter_index: M::Stored<PcoVec<Height, FilterByteIndex>>,
    /// The filters of each block's new Taproot output keys, end to end.
    #[traversable(hidden)]
    pub new_outputs_filter: M::Stored<BytesVec<FilterByteIndex, u8>>,
    /// Each block's end among the spent Taproot outputs' filter bytes.
    #[traversable(hidden)]
    pub end_spent_outputs_filter_index: M::Stored<PcoVec<Height, FilterByteIndex>>,
    /// The filters of each block's spends of Taproot outputs, end to end.
    #[traversable(hidden)]
    pub spent_outputs_filter: M::Stored<BytesVec<FilterByteIndex, u8>>,
}

impl Vecs {
    /// Data columns first, ends last: written in this order, a block's end never leads its data.
    fn stored_mut(&mut self) -> [&mut dyn AnyStoredVec; 8] {
        [
            &mut self.tx_index,
            &mut self.tweak_parity,
            &mut self.tweak_x,
            &mut self.new_outputs_filter,
            &mut self.spent_outputs_filter,
            &mut self.end_tweak_index,
            &mut self.end_new_outputs_filter_index,
            &mut self.end_spent_outputs_filter_index,
        ]
    }
}
