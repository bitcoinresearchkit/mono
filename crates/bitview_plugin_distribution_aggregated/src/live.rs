use bitview_plugin_distribution_common::state::cost_basis::PriceIndex;
use brk_types::{CentsCompact, Timestamp, Version};
use statedb::State;

/// One compact derived index; canonical origins remain owned by History.
pub(crate) struct LiveState {
    pub origins: State,
    pub index: PriceIndex<4>,
    pub prices: Vec<CentsCompact>,
    pub timestamps: Vec<Timestamp>,
    pub crossings: [usize; 3],
    pub version: (Version, Version, (u64, u64), Version),
}
