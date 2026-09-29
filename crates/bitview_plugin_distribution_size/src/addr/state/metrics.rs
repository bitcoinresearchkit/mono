use brk_types::Height;

use super::super::{
    AddrTypeToActivityCounts, AddrTypeToAddrCount, AddrVecs, ExposedAddrState, ReusedAddrState,
};

/// Runtime state for the address metrics pipeline.
#[derive(Debug, Default)]
pub struct AddrMetricsState {
    pub funded: AddrTypeToAddrCount,
    pub empty: AddrTypeToAddrCount,
    pub activity: AddrTypeToActivityCounts,
    pub reused: ReusedAddrState,
    pub respent: ReusedAddrState,
    pub exposed: ExposedAddrState,
}

impl AddrMetricsState {
    #[inline]
    pub fn reset_per_block(&mut self) {
        self.activity.reset();
        self.reused.reset_per_block();
        self.respent.reset_per_block();
    }
}

impl From<(&AddrVecs, Height)> for AddrMetricsState {
    #[inline]
    fn from((vecs, starting_height): (&AddrVecs, Height)) -> Self {
        Self {
            funded: AddrTypeToAddrCount::from((&vecs.funded.counts, starting_height)),
            empty: AddrTypeToAddrCount::from((&vecs.empty, starting_height)),
            activity: AddrTypeToActivityCounts::default(),
            reused: ReusedAddrState::from((&vecs.reused, starting_height)),
            respent: ReusedAddrState::from((&vecs.respent, starting_height)),
            exposed: ExposedAddrState::from((&vecs.exposed, starting_height)),
        }
    }
}
