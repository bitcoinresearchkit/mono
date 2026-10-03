use bitview_primitives::TypeIndex;
use brk_types::{Dollars, Sats};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Address statistics on the blockchain (confirmed transactions only)
///
/// Based on mempool.space's format with type_index extension.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddrChainStats {
    /// Current confirmed balance in satoshis
    pub balance: Sats,

    /// Total number of transaction outputs that funded this address
    #[schemars(example = 5)]
    pub funded_txo_count: u32,

    /// Total amount in satoshis received by this address across all funded outputs
    #[schemars(example = Sats::new(15007599040))]
    pub funded_txo_sum: Sats,

    /// Total number of transaction outputs spent from this address
    #[schemars(example = 5)]
    pub spent_txo_count: u32,

    /// Total amount in satoshis spent from this address
    #[schemars(example = Sats::new(15007599040))]
    pub spent_txo_sum: Sats,

    /// Total number of confirmed transactions involving this address
    #[schemars(example = 10)]
    pub tx_count: u32,

    /// Index of this address within its type on the blockchain
    #[schemars(example = TypeIndex::new(0))]
    pub type_index: TypeIndex,

    /// Realized price (average cost basis) in USD
    pub realized_price: Dollars,
}
