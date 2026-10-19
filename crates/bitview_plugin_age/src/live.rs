use brk_types::{Cents, Timestamp, Version};
use statedb::State;

use crate::{compute::PriceRangeMax, state::UTXOStates};

/// Writer-owned working state, retained only after the entire update succeeds.
pub(crate) struct LiveState {
    pub origins: State,
    pub states: UTXOStates,
    pub prices: Vec<Cents>,
    pub timestamps: Vec<Timestamp>,
    pub max: PriceRangeMax,
    /// The inputs' combined version.
    pub version: Version,
}
