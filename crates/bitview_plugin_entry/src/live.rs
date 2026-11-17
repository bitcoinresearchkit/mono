use bitview_cohort::{ByEntry, EntryPrice};
use bitview_distribution::state::{CoreRealizedState, MappedUTXOCohortState};
use brk_types::Version;

pub(crate) type CohortState = MappedUTXOCohortState<CoreRealizedState>;

/// Retained only after all entry metrics have completed successfully.
pub(crate) struct LiveState {
    pub(crate) cohorts: ByEntry<CohortState>,
    /// Each block's class, through the last one computed.
    pub(crate) entries: Vec<EntryPrice>,
    /// The hash of the last block computed, zero before the first: the chain the cohorts stand on.
    pub(crate) hash: [u8; 32],
    /// The inputs' combined version.
    pub(crate) version: Version,
}
