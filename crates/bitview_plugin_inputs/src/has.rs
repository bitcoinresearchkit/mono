use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the inputs plugin.
pub trait HasInputs<M: StorageMode> {
    fn inputs(&self) -> &Vecs<M>;
}
