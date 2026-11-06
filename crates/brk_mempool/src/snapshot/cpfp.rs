//! CPFP (Child Pays For Parent) walk over a `Snapshot`'s adjacency.
//!
//! Three walks over one component:
//! - `ancestors`: DFS up `parents` only.
//! - `descendants`: DFS down `children` only.
//! - cluster: the connected component over `parents ∪ children`, in one order whatever the seed,
//!   linearized for wire shape and the seed's chunk feerate (the snapshot's rates the same).

use brk_cpfp::Cluster;
use brk_error::Result;
use brk_types::{
    BlockHash, CpfpClusterChunk, CpfpClusterTxIndex, CpfpEntry, CpfpInfo, SigOps, Txid,
};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{SnapTx, Snapshot, TxIndex, cluster};
use crate::ReadOnlyState;

impl ReadOnlyState {
    /// CPFP info for a live mempool tx. Returns `None` when the tx
    /// isn't in the completed live publication. Requires the graph and live
    /// fields to share the same transaction revision and requested chain tip.
    pub fn cpfp_info(&self, txid: &Txid, tip: &BlockHash) -> Result<Option<CpfpInfo>> {
        let state = self.pool()?.resolved_at(tip)?;
        let snapshot = &state.graph;
        let Some(tx) = state.txs.get(txid) else {
            return Ok(None);
        };
        Ok(snapshot
            .idx_of_txid(txid)
            .map(|index| snapshot.cpfp_info_at(index, tx.total_sigop_cost)))
    }
}

impl Snapshot {
    /// From this snapshot's linearization of the seed's component: a request maps its chunks
    /// onto the members' order, never linearizing again.
    pub(crate) fn cpfp_info_at(&self, seed_idx: TxIndex, sigops: SigOps) -> CpfpInfo {
        let seed = &self.txs[seed_idx.as_usize()];
        let mut graph = Cluster::default();
        let members = cluster::load(&self.txs, &cluster::walk(&self.txs, seed_idx), &mut graph);
        let local: FxHashMap<TxIndex, u32> = members
            .iter()
            .enumerate()
            .map(|(i, &m)| (m, i as u32))
            .collect();
        let seed_local = local[&seed_idx];
        let chunks = self
            .chunks
            .component_chunks(self.chunk_of[seed_idx.as_usize()] as usize)
            .map(|c| {
                let mut txs: Vec<CpfpClusterTxIndex> = self
                    .chunks
                    .members(c)
                    .iter()
                    .map(|m| CpfpClusterTxIndex::from(local[m]))
                    .collect();
                txs.sort_unstable_by_key(|tx| u32::from(*tx));
                CpfpClusterChunk {
                    txs,
                    feerate: self.chunks.rate(c),
                }
            })
            .collect();
        CpfpInfo::new(
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.parents),
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.children),
            seed.fee,
            seed.weight,
            sigops,
            graph.project_from(
                CpfpClusterTxIndex::from(seed_local),
                |local| {
                    let t = &self.txs[members[local].as_usize()];
                    (t.txid, t.weight)
                },
                chunks,
            ),
        )
    }

    /// DFS from `seed` (exclusive) along `next`, lifted directly to wire-shape `CpfpEntry`s:
    /// the seed's ancestors or descendants, all of them (its component bounds them).
    fn collect_cpfp_entries(
        txs: &[SnapTx],
        seed: TxIndex,
        next: impl Fn(&SnapTx) -> &[TxIndex],
    ) -> Vec<CpfpEntry> {
        let mut visited: FxHashSet<TxIndex> = FxHashSet::default();
        visited.insert(seed);
        let mut out: Vec<CpfpEntry> = Vec::new();
        let mut stack: Vec<TxIndex> = next(&txs[seed.as_usize()]).to_vec();
        while let Some(idx) = stack.pop() {
            if !visited.insert(idx) {
                continue;
            }
            let t = &txs[idx.as_usize()];
            out.push(CpfpEntry::from(t));
            stack.extend(next(t).iter().copied());
        }
        out
    }
}
