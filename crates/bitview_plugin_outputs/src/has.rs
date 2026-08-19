use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the outputs plugin.
pub trait HasOutputs<M: StorageMode> {
    fn outputs(&self) -> &Vecs<M>;
}
