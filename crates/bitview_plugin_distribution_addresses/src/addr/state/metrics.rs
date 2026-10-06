use brk_types::{Height, OutputType};

use crate::addr::{
    AddrTypeToActivityCounts, AddrTypeToAddrCount, AddrVecs, ExposedAddrState, ReusedAddrState,
    add_type_delta,
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

    /// Adds the change to `output_type`'s values of a shard that started from `base`.
    pub fn add_type_delta(&mut self, shard: &Self, base: &Self, output_type: OutputType) {
        let Self {
            funded,
            empty,
            activity,
            reused,
            respent,
            exposed,
        } = shard;
        add_type_delta(&mut self.funded, funded, &base.funded, output_type);
        add_type_delta(&mut self.empty, empty, &base.empty, output_type);
        add_type_delta(&mut self.activity, activity, &base.activity, output_type);
        self.reused
            .add_type_delta(reused, &base.reused, output_type);
        self.respent
            .add_type_delta(respent, &base.respent, output_type);
        self.exposed
            .add_type_delta(exposed, &base.exposed, output_type);
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
