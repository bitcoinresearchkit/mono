use vecdb::StorageMode;

use crate::Vecs;

/// Provides access to the UTXO set plugin.
pub trait HasUtxoSet<M: StorageMode> {
    fn utxo_set(&self) -> &Vecs<M>;
}
