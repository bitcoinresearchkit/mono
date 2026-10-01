use vecdb::StorageMode;

use crate::Vecs;

pub trait HasDistributionUtxos<M: StorageMode> {
    fn distribution_utxos(&self) -> &Vecs<M>;
}
