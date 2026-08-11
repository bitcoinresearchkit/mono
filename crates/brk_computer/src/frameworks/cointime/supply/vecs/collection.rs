use brk_traversable::Traversable;
use brk_types::StoredF64;
use derive_more::{Deref, DerefMut};
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlock;

use super::LazyBaseVecs;

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub base: LazyBaseVecs,
    #[traversable(wrap = "active/in_loss", rename = "share")]
    pub active_supply_in_loss_share: PerBlock<StoredF64, M>,
}
