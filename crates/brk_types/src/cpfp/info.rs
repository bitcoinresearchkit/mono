#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{FeeRate, Sats, SigOps, VSize, Weight};

use super::{CpfpCluster, CpfpEntry};

/// CPFP (Child Pays For Parent) information for a transaction.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CpfpInfo {
    /// Ancestor transactions in the CPFP chain.
    pub ancestors: Vec<CpfpEntry>,
    /// Best (highest fee rate) descendant, if any.
    pub best_descendant: Option<CpfpEntry>,
    /// Descendant transactions in the CPFP chain.
    pub descendants: Vec<CpfpEntry>,
    /// Effective fee rate considering CPFP relationships (sat/vB).
    /// This is the seed's chunk feerate after lift-merging, i.e. the
    /// rate Core/mempool.space would surface for this tx.
    pub effective_fee_per_vsize: FeeRate,
    /// BIP-141 sigop cost for the seed tx (witness sigops count as 1,
    /// legacy and P2SH-redeem sigops count as 4).
    pub sigops: SigOps,
    /// Transaction fee (sats).
    pub fee: Sats,
    /// Virtual size of the seed tx (vbytes).
    pub vsize: VSize,
    /// Policy-adjusted virtual size: `max(vsize, sigops * 5)`.
    pub adjusted_vsize: VSize,
    /// Cluster the seed belongs to: full tx list, linearized chunks,
    /// and the seed's chunk index. Omitted when the seed has no
    /// ancestors and no descendants (matches mempool.space).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<CpfpCluster>,
}

impl CpfpInfo {
    /// The seed's info from its relatives, its fee, weight and sigops, and its cluster as served
    /// with its effective feerate.
    pub fn new(
        ancestors: Vec<CpfpEntry>,
        descendants: Vec<CpfpEntry>,
        fee: Sats,
        weight: Weight,
        sigops: SigOps,
        (cluster, effective_fee_per_vsize): (Option<CpfpCluster>, FeeRate),
    ) -> Self {
        let best_descendant = descendants
            .iter()
            .max_by_key(|entry| FeeRate::from((entry.fee, entry.weight)))
            .cloned();
        let vsize = VSize::from(weight);
        Self {
            ancestors,
            best_descendant,
            descendants,
            effective_fee_per_vsize,
            sigops,
            fee,
            vsize,
            adjusted_vsize: sigops.adjust_vsize(vsize),
            cluster,
        }
    }
}
