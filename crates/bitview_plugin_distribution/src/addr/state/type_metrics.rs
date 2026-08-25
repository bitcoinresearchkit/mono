use brk_types::{FundedAddrData, OutputType};

use crate::addr::{
    AddrMetricsState, AddrReceivePreState, AddrReceiveStatus, AddrSendPreState,
    BlockActivityCounts, ExposedAddrTypeState, ReusedAddrTypeState,
};

/// Mutable address metrics selected for one output type.
pub struct AddrTypeMetricsState<'a> {
    output_type: OutputType,
    funded: &'a mut u64,
    empty: &'a mut u64,
    activity: &'a mut BlockActivityCounts,
    reused: ReusedAddrTypeState<'a>,
    respent: ReusedAddrTypeState<'a>,
    exposed: ExposedAddrTypeState<'a>,
}

impl AddrMetricsState {
    #[inline]
    pub fn select(&mut self, output_type: OutputType) -> AddrTypeMetricsState<'_> {
        AddrTypeMetricsState {
            output_type,
            funded: self.funded.get_mut_unwrap(output_type),
            empty: self.empty.get_mut_unwrap(output_type),
            activity: self.activity.get_mut_unwrap(output_type),
            reused: self.reused.select(output_type),
            respent: self.respent.select(output_type),
            exposed: self.exposed.select(output_type),
        }
    }
}

impl AddrTypeMetricsState<'_> {
    #[inline]
    pub fn on_receive_applied(
        &mut self,
        status: AddrReceiveStatus,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        self.activity.receiving += 1;
        match status {
            AddrReceiveStatus::New => *self.funded += 1,
            AddrReceiveStatus::WasEmpty => {
                self.activity.reactivated += 1;
                *self.funded += 1;
                *self.empty -= 1;
            }
            AddrReceiveStatus::Tracked => {}
        }
        self.reused
            .on_receive_as_reused(addr_data, pre, output_count);
        self.respent
            .on_receive_as_respent(addr_data, pre, output_count);
        self.exposed
            .on_receive(self.output_type, addr_data, pre, status);
    }

    #[inline]
    pub fn on_send_applied(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if is_first_encounter {
            self.activity.sending += 1;
            if also_received {
                self.activity.bidirectional += 1;
            }
        }
        if will_be_empty {
            *self.funded -= 1;
            *self.empty += 1;
        }
        self.reused.on_send_as_reused(
            addr_data,
            pre,
            is_first_encounter,
            also_received,
            will_be_empty,
        );
        self.respent.on_send_as_respent(
            addr_data,
            pre,
            is_first_encounter,
            also_received,
            will_be_empty,
        );
        self.exposed
            .on_send(self.output_type, addr_data, pre, will_be_empty);
    }
}
