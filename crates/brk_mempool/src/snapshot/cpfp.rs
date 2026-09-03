//! CPFP (Child Pays For Parent) walk over a `Snapshot`'s adjacency.
//!
//! Three independent walks:
//! - `ancestors`: capped DFS up `parents` only.
//! - `descendants`: capped DFS down `children` only.
//! - cluster: connected component over `parents ∪ children`,
//!   linearized for wire shape and seed chunk feerate.

use brk_types::{
    CPFP_CHAIN_LIMIT, CpfpCluster, CpfpClusterTx, CpfpClusterTxIndex, CpfpEntry, CpfpInfo, FeeRate,
    SigOps, Txid, VSize, find_seed_chunk,
};
use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::Mempool;

use super::{Cluster, SnapTx, Snapshot, TxIndex};

impl Mempool {
    /// CPFP info for a live mempool tx. Returns `None` when the tx
    /// isn't in the live pool, so callers can fall through to the
    /// confirmed path. The snapshot can lag `state.txs` by up to one
    /// cycle: if the seed is in the snapshot but no longer in live
    /// state we return `None` rather than a half-stale report.
    pub fn cpfp_info(&self, txid: &Txid) -> Option<CpfpInfo> {
        let snapshot = self.snapshot();
        let seed_idx = snapshot.idx_of_txid(txid)?;
        let seed = snapshot.tx(seed_idx)?;

        let sigops = self.read().txs.get(txid)?.total_sigop_cost;

        Some(snapshot.cpfp_info_at(seed_idx, seed, sigops))
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
        let component = Cluster::walk(txs, seed_idx);
        if component.len() <= 1 {
            return (None, seed_per_tx_rate);
        }

        let (members, chunks) = Cluster::linearize(txs, &component);
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
        let local_of = Cluster::local_index(members);
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

#[cfg(test)]
mod tests {
    use brk_types::{FeeRate, Txid};

    use super::*;
    use crate::{
        state::TxEntry,
        test_support::{fake_entry_info, fake_tx, p2wpkh_script},
    };

    /// Insert a tx, optionally declaring parent dependencies for the
    /// snapshot builder's adjacency wire-up.
    fn insert_with_depends(
        mempool: &Mempool,
        seed: u8,
        fee: u64,
        vsize: u64,
        parents: &[Txid],
    ) -> Txid {
        let tx = fake_tx(seed, &[None], &[(p2wpkh_script(seed + 1), 1_234)]);
        let txid = tx.txid;
        let mut info = fake_entry_info(txid, fee, vsize);
        info.depends = parents.to_vec();
        let entry = TxEntry::new(&info, vsize, false);
        let mut state = mempool.test_state_lock().write();
        state.txs.insert(tx, entry);
        txid
    }

    #[test]
    fn singleton_cpfp_info_has_no_cluster() {
        let mempool = Mempool::for_test();
        let txid = insert_with_depends(&mempool, 0xB0, 10_000, 100, &[]);
        mempool.test_tick(&[txid], FeeRate::new(1.0));

        let info = mempool.cpfp_info(&txid).expect("tx is in mempool");
        assert!(info.cluster.is_none(), "singletons emit no cluster");
        assert!(info.ancestors.is_empty());
        assert!(info.descendants.is_empty());
        // Effective rate equals isolated rate when there's no package lift.
        let isolated = FeeRate::from((info.fee, info.vsize));
        assert_eq!(info.effective_fee_per_vsize, isolated);
    }

    #[test]
    fn two_tx_cpfp_cluster_has_both_members_and_lifted_rate() {
        let mempool = Mempool::for_test();
        let parent = insert_with_depends(&mempool, 0xB1, 100, 100, &[]);
        let child = insert_with_depends(&mempool, 0xB2, 1_900, 100, &[parent]);
        mempool.test_tick(&[parent, child], FeeRate::new(1.0));

        let parent_info = mempool.cpfp_info(&parent).unwrap();
        let cluster = parent_info.cluster.expect("two-tx cluster present");
        assert_eq!(cluster.txs.len(), 2);
        // Topological order: parent first.
        assert_eq!(cluster.txs[0].txid, parent);
        assert_eq!(cluster.txs[1].txid, child);
        // Child reports the parent as its only local parent.
        assert_eq!(cluster.txs[1].parents.len(), 1);
        // CPFP lift: parent's effective rate exceeds its isolated rate.
        let parent_isolated = FeeRate::from((parent_info.fee, parent_info.vsize));
        assert!(parent_info.effective_fee_per_vsize > parent_isolated);
        // Same package -> child's reported chunk rate matches parent's.
        let child_info = mempool.cpfp_info(&child).unwrap();
        assert_eq!(
            parent_info.effective_fee_per_vsize,
            child_info.effective_fee_per_vsize
        );
    }

    #[test]
    fn cpfp_ancestor_and_descendant_walks_are_directional() {
        // chain: A -> B -> C
        let mempool = Mempool::for_test();
        let a = insert_with_depends(&mempool, 0xB3, 100, 100, &[]);
        let b = insert_with_depends(&mempool, 0xB4, 100, 100, &[a]);
        let c = insert_with_depends(&mempool, 0xB5, 5_800, 100, &[b]);
        mempool.test_tick(&[a, b, c], FeeRate::new(1.0));

        // B sees A as an ancestor and C as a descendant.
        let info_b = mempool.cpfp_info(&b).unwrap();
        let ancestor_ids: Vec<_> = info_b.ancestors.iter().map(|e| e.txid).collect();
        let descendant_ids: Vec<_> = info_b.descendants.iter().map(|e| e.txid).collect();
        assert_eq!(ancestor_ids, vec![a]);
        assert_eq!(descendant_ids, vec![c]);
        // best_descendant picks the highest-rate descendant.
        assert_eq!(info_b.best_descendant.as_ref().map(|e| e.txid), Some(c));
    }

    #[test]
    fn cpfp_info_returns_none_for_unknown_txid() {
        let mempool = Mempool::for_test();
        mempool.test_tick(&[], FeeRate::new(1.0));
        assert!(mempool.cpfp_info(&Txid::COINBASE).is_none());
    }
}
