use brk_types::{FundedAddrData, OutputType, Sats};

use crate::addr::{AddrReceivePreState, AddrSendPreState, AddrTypeToSupply, ReusedAddrState};

/// Mutable reused/respent counters selected for one address type.
pub struct ReusedAddrTypeState<'a> {
    funded: &'a mut u64,
    total: &'a mut u64,
    supply: &'a mut Sats,
    output_events: &'a mut u64,
    input_events: &'a mut u64,
    active: &'a mut u64,
}

impl ReusedAddrState {
    #[inline]
    pub fn select(&mut self, output_type: OutputType) -> ReusedAddrTypeState<'_> {
        ReusedAddrTypeState {
            funded: self.funded.get_mut_unwrap(output_type),
            total: self.total.get_mut_unwrap(output_type),
            supply: self.supply.get_mut_unwrap(output_type),
            output_events: self.output_events.get_mut_unwrap(output_type),
            input_events: self.input_events.get_mut_unwrap(output_type),
            active: self.active.get_mut_unwrap(output_type),
        }
    }
}

impl ReusedAddrTypeState<'_> {
    #[inline]
    pub fn on_receive_as_reused(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        let is_now_reused = addr_data.is_reused();
        if is_now_reused && !pre.was_reused {
            *self.total += 1;
            *self.funded += 1;
        } else if pre.was_reused && !pre.was_funded {
            *self.funded += 1;
        }

        let skip_first = 1u32.saturating_sub(pre.prev_funded_txo_count.min(1));
        let reused_events = output_count.saturating_sub(skip_first);
        if reused_events > 0 {
            *self.output_events += u64::from(reused_events);
        }
        if is_now_reused {
            *self.active += 1;
        }
        AddrTypeToSupply::apply_delta(
            self.supply,
            pre.reused_contribution,
            addr_data.reused_supply_contribution(),
        );
    }

    #[inline]
    pub fn on_receive_as_respent(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        output_count: u32,
    ) {
        if pre.was_respent && !pre.was_funded {
            *self.funded += 1;
        }
        if pre.was_respent {
            *self.output_events += u64::from(output_count);
            *self.active += 1;
        }
        AddrTypeToSupply::apply_delta(
            self.supply,
            pre.respent_contribution,
            addr_data.respent_supply_contribution(),
        );
    }

    #[inline]
    pub fn on_send_as_reused(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if pre.was_reused {
            *self.input_events += 1;
        }
        if is_first_encounter && pre.was_reused && !also_received {
            *self.active += 1;
        }
        if will_be_empty && pre.was_reused {
            *self.funded -= 1;
        }
        AddrTypeToSupply::apply_delta(
            self.supply,
            pre.reused_contribution,
            addr_data.reused_supply_contribution(),
        );
    }

    #[inline]
    pub fn on_send_as_respent(
        &mut self,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        is_first_encounter: bool,
        also_received: bool,
        will_be_empty: bool,
    ) {
        if pre.was_respent {
            *self.input_events += 1;
        }

        let is_now_respent = addr_data.is_respent();
        if is_now_respent && !pre.was_respent {
            *self.total += 1;
            if !will_be_empty {
                *self.funded += 1;
            }
        }
        if (is_first_encounter && pre.was_respent && !also_received)
            || (is_now_respent && !pre.was_respent)
        {
            *self.active += 1;
        }
        if will_be_empty && pre.was_respent {
            *self.funded -= 1;
        }
        AddrTypeToSupply::apply_delta(
            self.supply,
            pre.respent_contribution,
            addr_data.respent_supply_contribution(),
        );
    }
}
