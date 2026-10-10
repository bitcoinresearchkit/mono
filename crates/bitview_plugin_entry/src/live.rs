use bitview_cohort::{ByEntry, EntryPrice};
use bitview_distribution::state::{CoreRealizedState, MappedUTXOCohortState};
use brk_types::{Cents, Timestamp, Version};
use statedb::State;

pub(crate) type CohortState = MappedUTXOCohortState<CoreRealizedState>;

/// Retained only after all entry metrics have completed successfully.
pub(crate) struct LiveState {
    pub(crate) origins: State,
    pub(crate) cohorts: ByEntry<CohortState>,
    pub(crate) entries: Vec<EntryPrice>,
    pub(crate) prices: Vec<Cents>,
    pub(crate) timestamps: Vec<Timestamp>,
    pub(crate) version: (Version, Version, Version, (u64, u64)),
}
