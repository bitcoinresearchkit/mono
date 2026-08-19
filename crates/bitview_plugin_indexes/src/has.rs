use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the indexes plugin.
pub trait HasIndexes<M: StorageMode> {
    fn indexes(&self) -> &Vecs<M>;
}
