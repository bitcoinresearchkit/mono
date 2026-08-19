use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the OP_RETURN plugin.
pub trait HasOpReturn<M: StorageMode> {
    fn op_return(&self) -> &Vecs<M>;
}
