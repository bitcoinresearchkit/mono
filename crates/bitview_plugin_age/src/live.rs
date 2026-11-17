use brk_types::Version;

use crate::{compute::PriceRangeMax, state::UTXOStates};

/// Writer-owned working state, kept only after a complete walk.
pub(crate) struct LiveState {
    pub states: UTXOStates,
    pub max: PriceRangeMax,
    /// The inputs' combined version.
    pub version: Version,
}
