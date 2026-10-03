use brk_types::{Sats, TxStatus, Txid, Vout};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Unspent transaction output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Utxo {
    /// Transaction ID of the UTXO
    pub txid: Txid,
    /// Output index
    pub vout: Vout,
    /// Confirmation status
    pub status: TxStatus,
    /// Output value in satoshis
    pub value: Sats,
}
