use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the mining plugin.
pub trait HasMining<M: StorageMode> {
    fn mining(&self) -> &Vecs<M>;
}
