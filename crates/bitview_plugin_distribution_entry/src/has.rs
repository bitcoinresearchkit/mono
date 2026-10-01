use crate::Vecs;
use vecdb::{Rw, StorageMode};

pub trait HasDistributionEntry<M: StorageMode = Rw> {
    fn distribution_entry(&self) -> &Vecs<M>;
}
