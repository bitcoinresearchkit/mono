use vecdb::StorageMode;

use crate::Vecs;

pub trait HasCointime<M: StorageMode> {
    fn cointime(&self) -> &Vecs<M>;
}
