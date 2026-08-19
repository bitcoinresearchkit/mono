use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the indicators plugin.
pub trait HasIndicators<M: StorageMode> {
    fn indicators(&self) -> &Vecs<M>;
}
