#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Position of a transaction inside a `CpfpCluster.txs` array. Cluster-local,
/// has no meaning outside the enclosing cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
pub struct CpfpClusterTxIndex(u32);

impl From<u32> for CpfpClusterTxIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<CpfpClusterTxIndex> for u32 {
    fn from(v: CpfpClusterTxIndex) -> Self {
        v.0
    }
}
