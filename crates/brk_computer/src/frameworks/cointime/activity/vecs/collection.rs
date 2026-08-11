use brk_traversable::Traversable;
use brk_types::StoredF64;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeRolling;

use super::DerivedVecs;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    pub coinblocks_created: PerBlockCumulativeRolling<StoredF64, M>,
    pub coinblocks_stored: PerBlockCumulativeRolling<StoredF64, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub derived: DerivedVecs<M>,
}
