#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Sats, Txid, Weight};

use super::CpfpClusterTxIndex;

/// One entry in a `CpfpCluster.txs` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CpfpClusterTx {
    pub txid: Txid,
    pub weight: Weight,
    pub fee: Sats,
    /// In-cluster parents of this tx.
    pub parents: Vec<CpfpClusterTxIndex>,
}
