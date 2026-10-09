use crate::Vecs;
use vecdb::{Rw, StorageMode};

pub trait HasEntry<M: StorageMode = Rw> {
    fn entry(&self) -> &Vecs<M>;
}
