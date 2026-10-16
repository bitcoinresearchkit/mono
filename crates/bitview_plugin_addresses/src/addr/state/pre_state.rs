use bitview_primitives::FundedAddressData;
use brk_types::{OutputType, Sats};
use derive_more::Deref;

/// Snapshot of [`FundedAddressData`]'s predicates and supply contributions, taken before a
/// spend or lost output mutates it.
#[derive(Debug)]
pub struct AddrPreState {
    pub was_reused: bool,
    pub was_respent: bool,
    pub was_pubkey_exposed: bool,
    pub exposed_contribution: Sats,
    pub reused_contribution: Sats,
    pub respent_contribution: Sats,
}

impl AddrPreState {
    #[inline]
    pub fn capture(addr_data: &FundedAddressData, output_type: OutputType) -> Self {
        Self {
            was_reused: addr_data.is_reused(),
            was_respent: addr_data.is_respent(),
            was_pubkey_exposed: addr_data.is_pubkey_exposed(output_type),
            exposed_contribution: addr_data.exposed_supply_contribution(output_type),
            reused_contribution: addr_data.reused_supply_contribution(),
            respent_contribution: addr_data.respent_supply_contribution(),
        }
    }
}

/// Snapshot of [`FundedAddressData`] taken before a receive mutates it.
#[derive(Debug, Deref)]
pub struct AddrReceivePreState {
    #[deref]
    pub pre: AddrPreState,
    pub was_funded: bool,
    pub prev_funded_txo_count: u32,
}

impl AddrReceivePreState {
    #[inline]
    pub fn capture(addr_data: &FundedAddressData, output_type: OutputType) -> Self {
        Self {
            pre: AddrPreState::capture(addr_data, output_type),
            was_funded: addr_data.is_funded(),
            prev_funded_txo_count: addr_data.funded_txo_count,
        }
    }
}
