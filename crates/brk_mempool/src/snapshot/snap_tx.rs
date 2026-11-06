use brk_types::{CpfpEntry, FeeRate, Sats, SigOps, Txid, VSize, Weight};
use smallvec::SmallVec;

use super::TxIndex;

/// Frozen per-tx view used by the snapshot. `chunk_rate` is the
/// linearized chunk feerate ([`brk_cpfp::Cluster::linearize`], run fresh
/// every snapshot). Singletons report `fee/vsize`. Parent/child
/// adjacency in `TxIndex` space, so CPFP queries are a pure walk over
/// `Snapshot.txs`.
#[derive(Clone, Debug)]
pub struct SnapTx {
    pub txid: Txid,
    /// What it pays: shown, and summed into a block's fees.
    pub fee: Sats,
    /// What Core's block builder ranks it by: `fee` plus the operator's `prioritisetransaction`
    /// delta, negative when that is. Linearization and packing use it, so its chunk rate is
    /// Core's (`getmempoolentry`'s `fees.chunk`).
    pub modified: i64,
    pub vsize: VSize,
    pub weight: Weight,
    /// Serialized tx size in bytes (witness + non-witness).
    pub size: u64,
    pub sigops: SigOps,
    pub chunk_rate: FeeRate,
    /// Direct parents in this snapshot; confirmed or absent parents are omitted.
    pub parents: SmallVec<[TxIndex; 2]>,
    pub children: SmallVec<[TxIndex; 4]>,
}

impl From<&SnapTx> for CpfpEntry {
    fn from(t: &SnapTx) -> Self {
        Self {
            txid: t.txid,
            weight: t.weight,
            fee: t.fee,
        }
    }
}
