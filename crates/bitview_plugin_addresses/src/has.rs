use vecdb::StorageMode;

use crate::Vecs;

pub trait HasAddresses<M: StorageMode> {
    fn addresses(&self) -> &Vecs<M>;
}
