use bitview_cohort::{ByEntry, EntryPrice};
use bitview_plugin_distribution_common::state::{
    CoreRealizedState, MappedUTXOCohortState, WithoutCapital,
};
use brk_types::{Cents, Timestamp, Version};
use statedb::State;

pub(crate) type CohortState = MappedUTXOCohortState<CoreRealizedState, WithoutCapital>;

/// Retained only after all entry metrics have completed successfully.
pub(crate) struct LiveState {
    pub origins: State,
    pub cohorts: ByEntry<CohortState>,
    pub entries: Vec<EntryPrice>,
    pub prices: Vec<Cents>,
    pub timestamps: Vec<Timestamp>,
    pub version: (Version, Version, Version, (u64, u64)),
}
