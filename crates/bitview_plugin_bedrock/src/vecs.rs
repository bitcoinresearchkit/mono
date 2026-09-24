pub mod compute;
pub mod import;

use bitview_plugin::{Plugin, PluginStorage};
use bitview_traversable::Traversable;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use super::{ModeVecs, Modes, STORAGE};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    modes: Modes<ModeVecs<M>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
