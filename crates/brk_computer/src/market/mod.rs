pub mod ath;
mod compute;
mod import;
pub mod lookback;
pub mod moving_average;
pub mod range;
pub mod returns;
pub mod technical;
pub mod volatility;

use bitview_plugin::{Plugin, PluginGate, PluginId};
use brk_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

pub use ath::Vecs as AthVecs;
pub use lookback::Vecs as LookbackVecs;
pub use moving_average::Vecs as MovingAverageVecs;
pub use range::Vecs as RangeVecs;
pub use returns::Vecs as ReturnsVecs;
pub use technical::Vecs as TechnicalVecs;
pub use volatility::Vecs as VolatilityVecs;

pub const DB_NAME: &str = "market";
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,
    pub ath: AthVecs<M>,
    pub lookback: LookbackVecs,
    pub returns: ReturnsVecs<M>,
    /// Population standard deviation of per-block trailing-24-hour spot-price
    /// returns over the named trailing monotonic-time window, multiplied by the
    /// square root of that window's day count.
    pub volatility: VolatilityVecs,
    pub range: RangeVecs<M>,
    pub moving_average: MovingAverageVecs<M>,
    pub technical: TechnicalVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        PluginId::new(DB_NAME)
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
