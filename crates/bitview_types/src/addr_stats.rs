use brk_types::{Address, AddressMempoolStats, OutputType, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::AddressChainStats;

/// Address information compatible with mempool.space API format.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddressStats {
    /// Bitcoin address, or the hex public key of a P2PK output
    #[schemars(
        example = "04678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5f"
    )]
    pub address: Address,

    /// Output script type
    pub address_type: OutputType,

    /// Statistics for confirmed transactions on the blockchain
    pub chain_stats: AddressChainStats,

    /// Statistics for unconfirmed transactions in the mempool
    pub mempool_stats: AddressMempoolStats,

    /// Total current balance in satoshis, including pending (unconfirmed) mempool changes
    pub balance: Sats,
}
