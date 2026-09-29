use brk_types::{FundedAddrData, OutputType, Sats};

/// Snapshot of [`FundedAddrData`] taken before a receive mutates it.
#[derive(Debug)]
pub struct AddrReceivePreState {
    pub was_funded: bool,
    pub was_reused: bool,
    pub was_respent: bool,
    pub was_pubkey_exposed: bool,
    pub prev_funded_txo_count: u32,
    pub exposed_contribution: Sats,
    pub reused_contribution: Sats,
    pub respent_contribution: Sats,
}

impl AddrReceivePreState {
    #[inline]
    pub fn capture(addr_data: &FundedAddrData, output_type: OutputType) -> Self {
        Self {
            was_funded: addr_data.is_funded(),
            was_reused: addr_data.is_reused(),
            was_respent: addr_data.is_respent(),
            was_pubkey_exposed: addr_data.is_pubkey_exposed(output_type),
            prev_funded_txo_count: addr_data.funded_txo_count,
            exposed_contribution: addr_data.exposed_supply_contribution(output_type),
            reused_contribution: addr_data.reused_supply_contribution(),
            respent_contribution: addr_data.respent_supply_contribution(),
        }
    }
}
