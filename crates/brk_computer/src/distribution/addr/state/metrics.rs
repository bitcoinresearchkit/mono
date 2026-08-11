use brk_types::{FundedAddrData, Height, OutputType};

use crate::distribution::block::TrackingStatus;

use super::super::{
    AddrTypeToActivityCounts, AddrTypeToAddrCount, AddrVecs, ExposedAddrState, ReusedAddrState,
};
use super::{AddrReceivePreState, AddrSendPreState};

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
    pub(crate) fn reset_per_block(&mut self) {
        self.activity.reset();
        self.reused.reset_per_block();
        self.respent.reset_per_block();
    }

    #[inline]
    pub(crate) fn on_receive_applied(
        &mut self,
        output_type: OutputType,
        status: TrackingStatus,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        let activity = self.activity.get_mut_unwrap(output_type);
        activity.receiving += 1;
        match status {
            TrackingStatus::New => {
                *self.funded.get_mut_unwrap(output_type) += 1;
            }
            TrackingStatus::WasEmpty => {
                activity.reactivated += 1;
                *self.funded.get_mut_unwrap(output_type) += 1;
                *self.empty.get_mut_unwrap(output_type) -= 1;
            }
            TrackingStatus::Tracked => {}
        }
        self.reused
            .on_receive_as_reused(output_type, addr_data, pre, output_count);
        self.respent
            .on_receive_as_respent(output_type, addr_data, pre, output_count);
        self.exposed.on_receive(output_type, addr_data, pre, status);
    }

    #[inline]
    pub(crate) fn on_send_applied(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if is_first_encounter {
            let activity = self.activity.get_mut_unwrap(output_type);
            activity.sending += 1;
            if also_received {
                activity.bidirectional += 1;
            }
        }
        if will_be_empty {
            *self.funded.get_mut_unwrap(output_type) -= 1;
            *self.empty.get_mut_unwrap(output_type) += 1;
        }
        self.reused.on_send_as_reused(
            output_type,
            addr_data,
            pre,
            is_first_encounter,
            also_received,
            will_be_empty,
        );
        self.respent.on_send_as_respent(
            output_type,
            addr_data,
            pre,
            is_first_encounter,
            also_received,
            will_be_empty,
        );
        self.exposed
            .on_send(output_type, addr_data, pre, will_be_empty);
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
