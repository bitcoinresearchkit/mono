use vecdb::StorageMode;

use crate::Vecs;

pub trait HasAge<M: StorageMode> {
    fn age(&self) -> &Vecs<M>;
}
