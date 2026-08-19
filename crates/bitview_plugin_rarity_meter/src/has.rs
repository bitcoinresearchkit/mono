use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the Rarity Meter plugin.
pub trait HasRarityMeter<M: StorageMode> {
    fn rarity_meter(&self) -> &Vecs<M>;
}
