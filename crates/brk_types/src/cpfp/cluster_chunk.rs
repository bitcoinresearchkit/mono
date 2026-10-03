use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRate;

use super::CpfpClusterTxIndex;

/// One SFL chunk inside a `CpfpCluster`. `txs` is in topological order
/// (matches `CpfpCluster.txs` ordering); the chunk's `feerate` is the
/// per-chunk SFL feerate and is the same for every tx in this chunk.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CpfpClusterChunk {
    pub txs: Vec<CpfpClusterTxIndex>,
    pub feerate: FeeRate,
}
