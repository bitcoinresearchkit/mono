use vecdb::StorageMode;

use crate::Vecs;

pub trait HasInputs<M: StorageMode> {
    fn inputs(&self) -> &Vecs<M>;
}
