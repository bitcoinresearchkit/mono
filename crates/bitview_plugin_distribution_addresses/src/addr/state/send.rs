use brk_types::{FundedAddrData, OutputType, Sats};

/// Snapshot of [`FundedAddrData`] taken before a spend mutates it.
#[derive(Debug)]
pub struct AddrSendPreState {
    pub was_reused: bool,
    pub was_respent: bool,
    pub was_pubkey_exposed: bool,
    pub exposed_contribution: Sats,
    pub reused_contribution: Sats,
    pub respent_contribution: Sats,
}

impl AddrSendPreState {
    #[inline]
    pub fn capture(addr_data: &FundedAddrData, output_type: OutputType) -> Self {
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
