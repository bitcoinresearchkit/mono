use vecdb::StorageMode;

use crate::Vecs;

pub trait HasPools<M: StorageMode> {
    fn pools(&self) -> &Vecs<M>;
}
