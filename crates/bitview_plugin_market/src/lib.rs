mod ath;
mod compute;
mod dependencies;
mod import;
mod moving_average;
mod power_law;
mod range;
mod returns;

pub use dependencies::Dependencies;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use ath::Vecs as AthVecs;
use moving_average::Vecs as MovingAverageVecs;
use power_law::Vecs as PowerLawVecs;
use range::Vecs as RangeVecs;
use returns::Returns;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("market"), Version::new(11));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    ath: AthVecs<M>,
    /// Bitcoin spot-price return over a trailing monotonic-time window: the
    /// represented block's price divided by the last price before the window,
    /// minus one. Positive values mean price increased and negative values mean
    /// it decreased. Null when that price is zero (before the first price) or
    /// the window starts at genesis.
    #[traversable(lead = "Returns: how much the price changed over the window.")]
    returns: Returns,
    range: RangeVecs<M>,
    #[traversable(flatten)]
    moving_average: MovingAverageVecs<M>,
    #[traversable(lead = "Power-law floors: published long-term lines of the price, and Sminston With's refit each day.")]
    power_law: PowerLawVecs<M>,
}
