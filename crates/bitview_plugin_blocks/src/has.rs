use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the blocks plugin.
pub trait HasBlocks<M: StorageMode> {
    fn blocks(&self) -> &Vecs<M>;
}
