use vecdb::StorageMode;

use crate::Vecs;

pub trait HasOutputs<M: StorageMode> {
    fn outputs(&self) -> &Vecs<M>;
}
