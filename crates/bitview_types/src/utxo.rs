use brk_types::{Sats, TxStatus, Txid, Vout};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Unspent transaction output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Utxo {
    pub txid: Txid,
    pub vout: Vout,
    pub status: TxStatus,
    /// Output value in satoshis
    pub value: Sats,
}
