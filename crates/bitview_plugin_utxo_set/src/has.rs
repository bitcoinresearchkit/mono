use vecdb::StorageMode;

use crate::Vecs;

pub trait HasUtxoSet<M: StorageMode> {
    fn utxo_set(&self) -> &Vecs<M>;
}
