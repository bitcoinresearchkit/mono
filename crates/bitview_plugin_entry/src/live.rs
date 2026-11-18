use bitview_cohort::{ByEntry, EntryPrice};
use bitview_distribution::state::{CoreRealizedState, SplitCohortState};
use bitview_primitives::CentsCompact;
use brk_types::{Sats, Version};

pub(crate) type CohortState = SplitCohortState<CoreRealizedState>;

/// Retained only after all entry metrics have completed successfully.
pub(crate) struct LiveState {
    pub(crate) cohorts: ByEntry<CohortState>,
    /// Each block's class, through the last one computed.
    pub(crate) entries: Vec<EntryPrice>,
    /// What each origin holds after the last block computed: the coins that change side when
    /// the spot passes its price.
    pub(crate) sats: Vec<Sats>,
    /// The spot the cohorts' splits stand at: zero until the first block computed.
    pub(crate) spot: CentsCompact,
    /// The hash of the last block computed, zero before the first: the chain the cohorts stand on.
    pub(crate) hash: [u8; 32],
    /// The inputs' combined version.
    pub(crate) version: Version,
}
