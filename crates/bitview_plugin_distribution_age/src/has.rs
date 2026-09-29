use vecdb::StorageMode;

use crate::Vecs;

pub trait HasDistributionAge<M: StorageMode> {
    fn distribution_age(&self) -> &Vecs<M>;
}
