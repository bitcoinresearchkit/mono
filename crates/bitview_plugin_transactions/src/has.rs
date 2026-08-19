use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the transactions plugin.
pub trait HasTransactions<M: StorageMode> {
    fn transactions(&self) -> &Vecs<M>;
}
