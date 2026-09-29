mod compute;
mod import;

use bitview_plugin::{Plugin, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::Replay;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use super::{Calibration, CumulativeBucket, ModeVecs, Modes, STORAGE};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    calibration: M::WriteOnly<Option<Calibration>>,
    #[traversable(skip)]
    replay: M::WriteOnly<Replay>,
    #[traversable(skip)]
    scratch: M::WriteOnly<Vec<CumulativeBucket>>,

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
