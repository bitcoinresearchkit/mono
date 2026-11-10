use brk_types::{Height, Timestamp, Txid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::BlockTxIndex;

/// An address's net satoshi change in one confirmed transaction. Zero-net activity is retained.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressTxSummary {
    pub txid: Txid,
    pub height: Height,
    /// Outputs funded to this exact script minus previous outputs spent from it, in satoshis.
    pub value: i64,
    /// Block header timestamp, in Unix seconds.
    pub time: Timestamp,
    /// Zero-based position in the confirming block (coinbase is zero).
    pub tx_position: BlockTxIndex,
}
