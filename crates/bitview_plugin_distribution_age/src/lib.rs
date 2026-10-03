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
pub use has::HasDistributionAge;

use bitview_cohort::AgeRange;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_urpd::AgeBoundsMetrics;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::{StoredF64, Version};
use vecdb::{Database, Rw, StorageMode};

use live::LiveState;
use metrics::CohortMetrics;

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("distribution_age"), Version::new(47));
pub const ID: PluginId = STORAGE.id();

/// Age-derived metrics and resident state, independent of address state.
#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,
    live: M::WriteOnly<Option<LiveState>>,
    pub cohorts: CohortMetrics<M>,
    #[traversable(wrap = "cohorts/urpd")]
    age_bounds: AgeBoundsMetrics<M>,
    #[traversable(wrap = "cointime/age_range")]
    pub coindays_created: AgeRange<PerBlockCumulativeRolling<StoredF64, M>>,
    #[traversable(wrap = "cointime/activity")]
    pub coinblocks_destroyed: PerBlockCumulativeRolling<StoredF64, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
