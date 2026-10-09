use crate::Vecs;
use vecdb::StorageMode;
pub trait HasHolders<M: StorageMode> {
    fn holders(&self) -> &Vecs<M>;
}
