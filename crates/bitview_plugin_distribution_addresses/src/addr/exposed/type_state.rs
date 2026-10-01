use brk_types::{FundedAddrData, OutputType, Sats};

use crate::addr::{
    AddrReceivePreState, AddrReceiveStatus, AddrSendPreState, AddrTypeToSupply, ExposedAddrState,
};

/// Mutable exposed-address counters selected for one address type.
pub struct ExposedAddrTypeState<'a> {
    funded: &'a mut u64,
    total: &'a mut u64,
    supply: &'a mut Sats,
}

impl ExposedAddrState {
    #[inline]
    pub fn select(&mut self, output_type: OutputType) -> ExposedAddrTypeState<'_> {
        ExposedAddrTypeState {
            funded: self.funded.get_mut_unwrap(output_type),
            total: self.total.get_mut_unwrap(output_type),
            supply: self.supply.get_mut_unwrap(output_type),
        }
    }
}

impl ExposedAddrTypeState<'_> {
    #[inline]
    pub fn on_receive(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        status: AddrReceiveStatus,
    ) {
        if !pre.was_funded && pre.was_pubkey_exposed {
            *self.funded += 1;
        }
        if output_type.pubkey_exposed_at_funding() && matches!(status, AddrReceiveStatus::New) {
            *self.total += 1;
        }
        AddrTypeToSupply::apply_delta(
            self.supply,
            pre.exposed_contribution,
            addr_data.exposed_supply_contribution(output_type),
        );
    }

    #[inline]
    pub fn on_send(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        will_be_empty: bool,
    ) {
        AddrTypeToSupply::apply_delta(
            self.supply,
            pre.exposed_contribution,
            addr_data.exposed_supply_contribution(output_type),
        );
        if !pre.was_pubkey_exposed {
            *self.total += 1;
            if !will_be_empty {
                *self.funded += 1;
            }
        }
        if will_be_empty && pre.was_pubkey_exposed {
            *self.funded -= 1;
        }
    }
}
