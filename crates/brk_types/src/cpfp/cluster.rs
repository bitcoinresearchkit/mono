#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CpfpClusterChunk, CpfpClusterTx};

/// CPFP cluster: the connected component the seed belongs to, plus its
/// SFL linearization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CpfpCluster {
    /// All txs in the cluster, in topological order (parents before children).
    pub txs: Vec<CpfpClusterTx>,
    /// SFL-emitted chunks ordered by descending feerate.
    pub chunks: Vec<CpfpClusterChunk>,
    /// Index into `chunks` of the chunk containing the seed tx.
    pub chunk_index: u32,
}
