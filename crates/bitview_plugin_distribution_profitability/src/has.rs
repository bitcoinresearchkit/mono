use crate::Vecs;
use vecdb::{Rw, StorageMode};

pub trait HasDistributionProfitability<M: StorageMode = Rw> {
    fn distribution_profitability(&self) -> &Vecs<M>;
}
