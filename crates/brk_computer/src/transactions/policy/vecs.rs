use brk_traversable::Traversable;
use brk_types::{StoredBool, TxIndex};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

mod count;

pub use count::CountVecs;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub count: CountVecs<M>,
    /// Whether the indexed transaction is classified as nonstandard under this
    /// approximation.
    pub is_nonstandard: M::Stored<EagerVec<PcoVec<TxIndex, StoredBool>>>,
}
