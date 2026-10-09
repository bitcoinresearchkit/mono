mod accounting_sources;
mod compute;
mod dependencies;
mod has;
mod import;
mod live;
mod metrics;
mod state;

pub use accounting_sources::AccountingSources;
pub use dependencies::Dependencies;
pub use has::HasAge;

use bitview_cohort::AgeRange;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::{CoinBlocks, CoinDays};
use bitview_traversable::Traversable;
use bitview_urpd::AgeBoundsMetrics;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use live::LiveState;
use metrics::CohortMetrics;

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("age"), Version::new(47));
pub const ID: PluginId = STORAGE.id();

/// Age-derived metrics and resident state, independent of address state.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    #[traversable(flatten)]
    pub cohorts: CohortMetrics<M>,
    age_bounds: AgeBoundsMetrics<M>,
    pub coindays_created: AgeRange<PerBlockCumulativeRolling<CoinDays, M>>,
    pub coinblocks_destroyed: PerBlockCumulativeRolling<CoinBlocks, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
