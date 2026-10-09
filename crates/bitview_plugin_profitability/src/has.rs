use crate::Vecs;
use vecdb::{Rw, StorageMode};

pub trait HasProfitability<M: StorageMode = Rw> {
    fn profitability(&self) -> &Vecs<M>;
}
