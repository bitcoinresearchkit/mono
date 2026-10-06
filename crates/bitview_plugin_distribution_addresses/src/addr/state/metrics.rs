use brk_types::{Height, OutputType};

use super::super::{
    AddrTypeToActivityCounts, AddrTypeToAddrCount, AddrVecs, ExposedAddrState, ReusedAddrState,
};

/// Runtime state for the address metrics pipeline.
#[derive(Debug, Default, Clone)]
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

    /// Takes `output_type`'s values from `other`, which processed that type.
    pub fn copy_type(&mut self, other: &Self, output_type: OutputType) {
        let Self {
            funded,
            empty,
            activity,
            reused,
            respent,
            exposed,
        } = other;
        *self.funded.get_mut_unwrap(output_type) = *funded.get_unwrap(output_type);
        *self.empty.get_mut_unwrap(output_type) = *empty.get_unwrap(output_type);
        *self.activity.get_mut_unwrap(output_type) = *activity.get_unwrap(output_type);
        self.reused.copy_type(reused, output_type);
        self.respent.copy_type(respent, output_type);
        self.exposed.copy_type(exposed, output_type);
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
