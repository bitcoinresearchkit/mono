use derive_more::{From, Into};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Position of a transaction inside a `CpfpCluster.txs` array. Cluster-local,
/// has no meaning outside the enclosing cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, From, Into)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
pub struct CpfpClusterTxIndex(u32);
