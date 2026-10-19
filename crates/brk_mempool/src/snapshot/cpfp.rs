//! CPFP (Child Pays For Parent) walk over a `Snapshot`'s adjacency.
//!
//! Three independent walks:
//! - `ancestors`: capped DFS up `parents` only.
//! - `descendants`: capped DFS down `children` only.
//! - cluster: connected component over `parents ∪ children`,
//!   linearized for wire shape and seed chunk feerate.

use brk_cpfp::Cluster;
use brk_error::Result;
use brk_types::{
    BlockHash, CPFP_CHAIN_LIMIT, CpfpClusterTxIndex, CpfpEntry, CpfpInfo, SigOps, Txid,
};
use rustc_hash::{FxBuildHasher, FxHashSet};

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
    fn cpfp_info_at(&self, seed_idx: TxIndex, sigops: SigOps) -> CpfpInfo {
        let seed = &self.txs[seed_idx.as_usize()];
        let mut graph = Cluster::default();
        let members = cluster::load(&self.txs, &cluster::walk(&self.txs, seed_idx), &mut graph);
        let seed_local = members
            .iter()
            .position(|&i| i == seed_idx)
            .expect("the seed is in its own cluster") as u32;
        CpfpInfo::new(
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.parents),
            Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.children),
            seed.fee,
            seed.weight,
            sigops,
            graph.project(CpfpClusterTxIndex::from(seed_local), |local| {
                let t = &self.txs[members[local].as_usize()];
                (t.txid, t.weight)
            }),
        )
    }

    /// Capped DFS from `seed` (exclusive) along `next`, lifted directly
    /// to wire-shape `CpfpEntry`s. Used for both ancestor and descendant
    /// walks.
    fn collect_cpfp_entries(
        txs: &[SnapTx],
        seed: TxIndex,
        next: impl Fn(&SnapTx) -> &[TxIndex],
    ) -> Vec<CpfpEntry> {
        let seed_node = &txs[seed.as_usize()];
        let mut visited: FxHashSet<TxIndex> =
            FxHashSet::with_capacity_and_hasher(CPFP_CHAIN_LIMIT + 1, FxBuildHasher);
        visited.insert(seed);
        let mut out: Vec<CpfpEntry> = Vec::with_capacity(CPFP_CHAIN_LIMIT);
        let mut stack: Vec<TxIndex> = next(seed_node).to_vec();
        while let Some(idx) = stack.pop() {
            if out.len() >= CPFP_CHAIN_LIMIT {
                break;
            }
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
