use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the supply plugin.
pub trait HasSupply<M: StorageMode> {
    fn supply(&self) -> &Vecs<M>;
}
