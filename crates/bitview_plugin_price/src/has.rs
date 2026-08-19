use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the price plugin.
pub trait HasPrice<M: StorageMode> {
    fn price(&self) -> &Vecs<M>;
}
