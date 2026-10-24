#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::FeeRate;

use super::{CpfpCluster, CpfpEntry, CpfpInfo};

/// A transaction's CPFP (Child Pays For Parent) answer, in mempool.space's shapes, told apart by
/// their fields (untagged on the wire): a mempool transaction's package, a confirmed transaction's
/// package among its block's transactions, or `{"ancestors":[]}` for an unknown transaction or a
/// confirmed one without relatives in its block.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged)]
pub enum Cpfp {
    Mempool(CpfpInfo),
    Confirmed(ConfirmedCpfp),
    Empty(EmptyCpfp),
}

impl Cpfp {
    pub const EMPTY: Self = Self::Empty(EmptyCpfp { ancestors: [] });
}

/// CPFP of a confirmed transaction with relatives in its block: its package there.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedCpfp {
    pub ancestors: Vec<CpfpEntry>,
    pub descendants: Vec<CpfpEntry>,
    /// Feerate of the chunk holding the transaction when its block's relatives are linearized
    /// (sat/vB).
    pub effective_fee_per_vsize: FeeRate,
    /// The transaction's relatives in its block.
    pub cluster: CpfpCluster,
}

/// CPFP of a transaction without known relatives: `{"ancestors":[]}`.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct EmptyCpfp {
    pub ancestors: [CpfpEntry; 0],
}
