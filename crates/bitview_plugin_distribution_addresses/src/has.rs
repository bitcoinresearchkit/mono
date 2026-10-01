use vecdb::StorageMode;

use crate::Vecs;

pub trait HasDistributionAddresses<M: StorageMode> {
    fn distribution_addresses(&self) -> &Vecs<M>;
}
