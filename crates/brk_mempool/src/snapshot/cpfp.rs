//! CPFP (Child Pays For Parent) walk over a `Snapshot`'s adjacency.
//!
//! Three independent walks:
//! - `ancestors`: capped DFS up `parents` only.
//! - `descendants`: capped DFS down `children` only.
//! - cluster: connected component over `parents ∪ children`,
//!   linearized for wire shape and seed chunk feerate.

use brk_error::Result;
use brk_types::{
    BlockHash, CPFP_CHAIN_LIMIT, CpfpCluster, CpfpClusterTx, CpfpClusterTxIndex, CpfpEntry,
    CpfpInfo, FeeRate, SigOps, Txid, VSize, find_seed_chunk,
};
use rustc_hash::{FxBuildHasher, FxHashSet};

use super::{SnapTx, Snapshot, TxIndex, cluster};
use crate::ReadOnlyState;

impl ReadOnlyState {
    /// CPFP info for a live mempool tx. Returns `None` when the tx
    /// isn't in the completed live publication. Requires the graph and live
    /// fields to share the same transaction revision and requested chain tip.
    pub fn cpfp_info(&self, txid: &Txid, tip: &BlockHash) -> Result<Option<CpfpInfo>> {
        let state = self.pool()?;
        state.ensure_resolved_at(tip)?;
        let snapshot = &state.graph;
        let Some(tx) = state.txs.get(txid) else {
            return Ok(None);
        };
        Ok(snapshot.idx_of_txid(txid).and_then(|index| {
            snapshot
                .tx(index)
                .map(|seed| snapshot.cpfp_info_at(index, seed, tx.total_sigop_cost))
        }))
    }
}

impl Snapshot {
    fn cpfp_info_at(&self, seed_idx: TxIndex, seed: &SnapTx, sigops: SigOps) -> CpfpInfo {
        let ancestors = Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.parents);
        let descendants = Self::collect_cpfp_entries(&self.txs, seed_idx, |t| &t.children);
        let best_descendant = descendants
            .iter()
            .max_by_key(|e| FeeRate::from((e.fee, e.weight)))
            .cloned();

        let (cluster, effective_fee_per_vsize) =
            Self::build_cpfp_cluster(&self.txs, seed_idx, seed);
        let vsize = VSize::from(seed.weight);

        CpfpInfo {
            ancestors,
            best_descendant,
            descendants,
            effective_fee_per_vsize,
            sigops,
            fee: seed.fee,
            vsize,
            adjusted_vsize: sigops.adjust_vsize(vsize),
            cluster,
        }
    }

    /// Capped DFS from `seed` (exclusive) along `next`, lifted directly
    /// to wire-shape `CpfpEntry`s. Used for both ancestor and descendant
    /// walks.
    fn collect_cpfp_entries(
        txs: &[SnapTx],
        seed: TxIndex,
        next: impl Fn(&SnapTx) -> &[TxIndex],
    ) -> Vec<CpfpEntry> {
        let Some(seed_node) = txs.get(seed.as_usize()) else {
            return Vec::new();
        };
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
            if let Some(t) = txs.get(idx.as_usize()) {
                out.push(CpfpEntry::from(t));
                stack.extend(next(t).iter().copied());
            }
        }
        out
    }

    /// Wire-shape `CpfpCluster` plus the seed's chunk feerate. Members
    /// are the connected component of the seed in the dependency graph,
    /// topologically ordered (parents before children) so wire indices
    /// and chunk-internal ordering are valid for client-side
    /// reconstruction. Returns `(None, seed_per_tx_rate)` for singletons
    /// (matches mempool.space, which omits `cluster` when no relations
    /// exist).
    fn build_cpfp_cluster(
        txs: &[SnapTx],
        seed_idx: TxIndex,
        seed: &SnapTx,
    ) -> (Option<CpfpCluster>, FeeRate) {
        let seed_per_tx_rate = FeeRate::from((seed.fee, seed.vsize));
        let component = cluster::walk(txs, seed_idx);
        if component.len() <= 1 {
            return (None, seed_per_tx_rate);
        }

        let (members, chunks) = cluster::linearize(txs, &component);
        let cluster_txs = Self::wire_cluster_members(txs, &members);
        let seed_local = CpfpClusterTxIndex::from(
            members
                .iter()
                .position(|&i| i == seed_idx)
                .map_or(0, |p| p as u32),
        );
        let (chunk_index, seed_chunk_rate) = find_seed_chunk(&chunks, seed_local, seed_per_tx_rate);

        (
            Some(CpfpCluster {
                txs: cluster_txs,
                chunks,
                chunk_index,
            }),
            seed_chunk_rate,
        )
    }

    /// Materialize wire-shape `CpfpClusterTx`s for every topo-ordered
    /// member with parent edges remapped to local indices.
    fn wire_cluster_members(txs: &[SnapTx], members: &[TxIndex]) -> Vec<CpfpClusterTx> {
        let local_of = cluster::local_index(members);
        members
            .iter()
            .map(|&idx| {
                let t = &txs[idx.as_usize()];
                CpfpClusterTx {
                    txid: t.txid,
                    weight: t.weight,
                    fee: t.fee,
                    parents: t
                        .parents
                        .iter()
                        .filter_map(|p| local_of.get(p).copied())
                        .collect(),
                }
            })
            .collect()
    }
}
