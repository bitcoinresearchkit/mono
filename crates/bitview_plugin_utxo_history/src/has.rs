use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to published UTXO history metrics.
pub trait HasUtxoHistory<M: StorageMode> {
    fn utxo_history(&self) -> &Vecs<M>;
}
