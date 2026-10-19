use vecdb::StorageMode;

use crate::Vecs;

pub trait HasMining<M: StorageMode> {
    fn mining(&self) -> &Vecs<M>;
}
