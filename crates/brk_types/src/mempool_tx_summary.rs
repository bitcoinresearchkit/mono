#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Sats, Transaction, Txid, VSize};

/// An unconfirmed transaction in esplora's light shape, as `/api/mempool/recent` and the projected
/// mempool blocks list them: the whole body is at `GET /api/tx/{txid}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MempoolTxSummary {
    pub txid: Txid,
    /// Transaction fee (sats)
    pub fee: Sats,
    /// Virtual size (vbytes)
    pub vsize: VSize,
    /// Total output value (sats)
    pub value: Sats,
}

impl From<&Transaction> for MempoolTxSummary {
    fn from(tx: &Transaction) -> Self {
        Self {
            txid: tx.txid,
            fee: tx.fee,
            vsize: tx.vsize(),
            value: tx.output.iter().map(|o| o.value).sum(),
        }
    }
}
