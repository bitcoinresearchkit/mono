use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the distribution plugin.
pub trait HasDistribution<M: StorageMode> {
    fn distribution(&self) -> &Vecs<M>;
}
