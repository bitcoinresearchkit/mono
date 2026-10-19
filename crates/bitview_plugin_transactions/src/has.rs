use vecdb::StorageMode;

use crate::Vecs;

pub trait HasTransactions<M: StorageMode> {
    fn transactions(&self) -> &Vecs<M>;
}
