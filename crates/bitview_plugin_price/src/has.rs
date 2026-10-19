use vecdb::StorageMode;

use crate::Vecs;

pub trait HasPrice<M: StorageMode> {
    fn price(&self) -> &Vecs<M>;
}
