use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the Cointime plugin.
pub trait HasCointime<M: StorageMode> {
    fn cointime(&self) -> &Vecs<M>;
}
