use vecdb::StorageMode;

use crate::Vecs;

pub trait HasBlocks<M: StorageMode> {
    fn blocks(&self) -> &Vecs<M>;
}
