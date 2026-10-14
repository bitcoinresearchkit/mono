mod ath;
mod compute;
mod dependencies;
mod import;
mod moving_average;
mod range;
mod returns;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use ath::Vecs as AthVecs;
use moving_average::Vecs as MovingAverageVecs;
use range::Vecs as RangeVecs;
use returns::Returns;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("market"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    ath: AthVecs<M>,
    /// Bitcoin spot-price return from the first block in a trailing
    /// monotonic-time window through the represented block: represented-block
    /// price divided by the window's starting price, minus one. Positive values
    /// mean price increased and negative values mean it decreased. Null when the
    /// window's starting price is zero.
    returns: Returns,
    range: RangeVecs<M>,
    #[traversable(flatten)]
    moving_average: MovingAverageVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
