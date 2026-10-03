mod ath;
mod compute;
mod dependencies;
mod has;
mod import;
mod lookback;
mod moving_average;
mod range;
mod returns;
mod technical;
mod volatility;

pub use dependencies::Dependencies;
pub use has::HasMarket;
pub use moving_average::Vecs as MovingAverageVecs;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use ath::Vecs as AthVecs;
use lookback::Vecs as LookbackVecs;
use range::Vecs as RangeVecs;
use returns::Vecs as ReturnsVecs;
use technical::Vecs as TechnicalVecs;
use volatility::Vecs as VolatilityVecs;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("market"), Version::new(9));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    ath: AthVecs<M>,
    lookback: LookbackVecs,
    returns: ReturnsVecs<M>,
    /// Population standard deviation of per-block trailing-24-hour spot-price
    /// returns over a trailing monotonic-time window, multiplied by the
    /// square root of that window's day count. Larger values mean Bitcoin's
    /// daily returns varied more within the window; this measures dispersion,
    /// not price direction.
    pub volatility: VolatilityVecs,
    range: RangeVecs<M>,
    pub moving_average: MovingAverageVecs<M>,
    technical: TechnicalVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
