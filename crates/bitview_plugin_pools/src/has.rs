use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the pools plugin.
pub trait HasPools<M: StorageMode> {
    fn pools(&self) -> &Vecs<M>;
}
