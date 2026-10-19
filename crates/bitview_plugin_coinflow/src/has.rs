use vecdb::StorageMode;

use crate::Vecs;

pub trait HasCoinflow<M: StorageMode> {
    fn coinflow(&self) -> &Vecs<M>;
}
