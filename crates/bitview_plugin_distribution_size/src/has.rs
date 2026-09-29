use vecdb::StorageMode;

use crate::Vecs;

pub trait HasDistributionSize<M: StorageMode> {
    fn distribution_size(&self) -> &Vecs<M>;
}
