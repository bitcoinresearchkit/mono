use brk_types::{FundedAddrData, Height, OutputType};

use crate::distribution::{
    addr::{AddrReceivePreState, AddrSendPreState, AddrTypeToAddrCount, AddrTypeToSupply},
    block::TrackingStatus,
};

use super::ExposedAddrVecs;

/// Runtime running totals for exposed-addr tracking. Mirrors the persistent
/// fields of [`ExposedAddrVecs`]: funded count, total count, funded supply.
/// Recovered from disk at the start of a block-loop run.
#[derive(Debug, Default)]
pub struct ExposedAddrState {
    pub funded: AddrTypeToAddrCount,
    pub total: AddrTypeToAddrCount,
    pub supply: AddrTypeToSupply,
}

impl ExposedAddrState {
    /// Apply exposed-addr updates for a received output, AFTER the receive
    /// has mutated `addr_data`. `pre` is the snapshot taken before the mutation.
    #[inline]
    pub fn on_receive(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrReceivePreState,
        status: TrackingStatus,
    ) {
        // Pubkey-exposure state is unchanged by a receive, so `pre.was_pubkey_exposed`
        // equals the post-receive value.
        if !pre.was_funded && pre.was_pubkey_exposed {
            *self.funded.get_mut_unwrap(output_type) += 1;
        }
        // Total for pk-exposed-at-funding types fires here on first receive.
        // Other types fire on first spend in [`Self::on_send`].
        if output_type.pubkey_exposed_at_funding() && matches!(status, TrackingStatus::New) {
            *self.total.get_mut_unwrap(output_type) += 1;
        }
        let after = addr_data.exposed_supply_contribution(output_type);
        self.supply
            .apply_delta(output_type, pre.exposed_contribution, after);
    }

    /// Apply exposed-addr updates for a spent UTXO, AFTER the send has mutated
    /// `addr_data`. `pre` is the snapshot taken before the mutation.
    #[inline]
    pub fn on_send(
        &mut self,
        output_type: OutputType,
        addr_data: &FundedAddrData,
        pre: &AddrSendPreState,
        will_be_empty: bool,
    ) {
        let after = addr_data.exposed_supply_contribution(output_type);
        self.supply
            .apply_delta(output_type, pre.exposed_contribution, after);
        // First-ever pubkey exposure. Non-pk-exposed types fire on first spend.
        // Pk-exposed types never fire here (was already exposed at first receive).
        if !pre.was_pubkey_exposed {
            *self.total.get_mut_unwrap(output_type) += 1;
            if !will_be_empty {
                *self.funded.get_mut_unwrap(output_type) += 1;
            }
        }
        // Leaving the funded exposed set: was in it iff pubkey was exposed pre-spend.
        if will_be_empty && pre.was_pubkey_exposed {
            *self.funded.get_mut_unwrap(output_type) -= 1;
        }
    }
}

impl From<(&ExposedAddrVecs, Height)> for ExposedAddrState {
    #[inline]
    fn from((vecs, starting_height): (&ExposedAddrVecs, Height)) -> Self {
        Self {
            funded: AddrTypeToAddrCount::from((&vecs.count.funded, starting_height)),
            total: AddrTypeToAddrCount::from((&vecs.count.total, starting_height)),
            supply: AddrTypeToSupply::from((&vecs.supply, starting_height)),
        }
    }
}
