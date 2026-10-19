#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRate;

use super::CpfpClusterTxIndex;

/// One linearized chunk inside a `CpfpCluster`. `txs` follows `CpfpCluster.txs` order (topological);
/// `feerate` is the chunk's, shared by all its transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CpfpClusterChunk {
    pub txs: Vec<CpfpClusterTxIndex>,
    pub feerate: FeeRate,
}
