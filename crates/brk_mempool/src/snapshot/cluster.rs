//! Cluster primitives over `SnapTx` adjacency: connected-component
//! discovery, topo-sort, and the glue to Single Fee Linearization
//! ([`brk_types::linearize`], shared with bitview_query's confirmed-cpfp).

use std::collections::VecDeque;

use brk_types::{ChunkInput, CpfpClusterChunk, CpfpClusterTxIndex, linearize};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use smallvec::SmallVec;

use super::{SnapTx, TxIndex};

pub struct Cluster;

/// Matches Bitcoin Core 31's `MAX_CLUSTER_COUNT_LIMIT`.
const MAX_CLUSTER: usize = 64;

impl Cluster {
    /// Capped DFS over the undirected dependency graph (`parents ∪
    /// children`) starting from `seed`. Returns the connected component
    /// truncated to `MAX_CLUSTER`, with `seed` at index 0.
    pub fn walk(txs: &[SnapTx], seed: TxIndex) -> Vec<TxIndex> {
        if txs.get(seed.as_usize()).is_none() {
            return Vec::new();
        }
        let mut visited: FxHashSet<TxIndex> =
            FxHashSet::with_capacity_and_hasher(MAX_CLUSTER, FxBuildHasher);
        visited.insert(seed);
        let mut out: Vec<TxIndex> = Vec::with_capacity(MAX_CLUSTER);
        out.push(seed);
        let mut stack: Vec<TxIndex> = vec![seed];
        while let Some(idx) = stack.pop() {
            let Some(t) = txs.get(idx.as_usize()) else {
                continue;
            };
            for &n in t.parents.iter().chain(t.children.iter()) {
                if out.len() >= MAX_CLUSTER {
                    return out;
                }
                if visited.insert(n) {
                    out.push(n);
                    stack.push(n);
                }
            }
        }
        out
    }

    /// Linearize the connected component into chunks. Topo-sorts members,
    /// remaps parent edges to cluster-local indices, and runs SFL. Returns
    /// `(members, chunks)` where `members` is the topo-ordered `TxIndex`
    /// list and `chunks[*].txs` are local indices into `members`. Callers
    /// must filter singletons before calling - the singleton's `chunk_rate`
    /// is `fee/vsize`, set elsewhere.
    pub fn linearize(
        txs: &[SnapTx],
        component: &[TxIndex],
    ) -> (Vec<TxIndex>, Vec<CpfpClusterChunk>) {
        let members = Self::topo_sort(txs, component);
        let local_of = Self::local_index(&members);
        let parents_local: Vec<SmallVec<[CpfpClusterTxIndex; 2]>> = members
            .iter()
            .map(|idx| {
                txs[idx.as_usize()]
                    .parents
                    .iter()
                    .filter_map(|p| local_of.get(p).copied())
                    .collect()
            })
            .collect();
        let inputs: Vec<ChunkInput<'_>> = members
            .iter()
            .zip(&parents_local)
            .map(|(idx, ps)| {
                let t = &txs[idx.as_usize()];
                ChunkInput {
                    fee: t.fee,
                    vsize: t.vsize,
                    parents: ps.as_slice(),
                }
            })
            .collect();
        let chunks = linearize(&inputs);
        (members, chunks)
    }

    /// `members[i]`'s wire index, keyed by snapshot `TxIndex`. Built once
    /// so per-tx parent edges can be remapped without a linear scan.
    pub fn local_index(members: &[TxIndex]) -> FxHashMap<TxIndex, CpfpClusterTxIndex> {
        members
            .iter()
            .enumerate()
            .map(|(i, &idx)| (idx, CpfpClusterTxIndex::from(i as u32)))
            .collect()
    }

    /// Kahn's topological sort over the connected component, restricted to
    /// in-cluster parent edges. Returns members in an order where every tx
    /// follows all its in-cluster parents.
    fn topo_sort(txs: &[SnapTx], component: &[TxIndex]) -> Vec<TxIndex> {
        let n = component.len();
        let pos: FxHashMap<TxIndex, usize> =
            component.iter().enumerate().map(|(i, &x)| (x, i)).collect();
        let mut indeg: Vec<u32> = vec![0; n];
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (i, &idx) in component.iter().enumerate() {
            let Some(t) = txs.get(idx.as_usize()) else {
                continue;
            };
            indeg[i] = t.parents.iter().filter(|p| pos.contains_key(p)).count() as u32;
            for &c in t.children.iter() {
                if let Some(&ci) = pos.get(&c) {
                    children[i].push(ci);
                }
            }
        }
        let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut out: Vec<TxIndex> = Vec::with_capacity(n);
        while let Some(i) = queue.pop_front() {
            out.push(component[i]);
            for &c in &children[i] {
                indeg[c] -= 1;
                if indeg[c] == 0 {
                    queue.push_back(c);
                }
            }
        }
        out
    }
}
