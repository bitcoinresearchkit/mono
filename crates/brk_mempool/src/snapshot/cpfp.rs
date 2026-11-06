//! CPFP (Child Pays For Parent) walk over a `Snapshot`'s adjacency.
//!
//! Three walks over one component:
//! - `ancestors`: DFS up `parents` only.
//! - `descendants`: DFS down `children` only.
//! - cluster: the connected component over `parents ∪ children`, in one order whatever the seed,
//!   linearized for wire shape and the seed's chunk feerate (the snapshot's rates the same).

use brk_cpfp::Cluster;
use brk_error::Result;
use brk_types::{BlockHash, CpfpClusterTxIndex, CpfpEntry, CpfpInfo, SigOps, Txid};
use rustc_hash::FxHashSet;

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
