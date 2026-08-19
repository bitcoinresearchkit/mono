use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the Coinflow plugin.
pub trait HasCoinflow<M: StorageMode> {
    fn coinflow(&self) -> &Vecs<M>;
}
