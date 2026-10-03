use bitview_primitives::StoredBool;
use bitview_traversable::Traversable;
use brk_types::TxIndex;
use derive_more::{Deref, DerefMut};
use vecdb::{EagerVec, PcoVec, Rw, StorageMode};

use super::{CountVecs, Flags};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub count: CountVecs<M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub flags: Flags<M::Stored<EagerVec<PcoVec<TxIndex, StoredBool>>>>,
}
