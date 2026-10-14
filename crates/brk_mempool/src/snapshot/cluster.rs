//! Cluster primitives over `SnapTx` adjacency: connected-component
//! discovery, topo-sort, and loading into a [`brk_cpfp::Cluster`] (shared
//! with bitview_query's confirmed CPFP and the transactions plugin).

use std::collections::VecDeque;

use brk_cpfp::Cluster;
use brk_types::CpfpClusterTxIndex;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use super::{SnapTx, TxIndex};

/// Matches Bitcoin Core 31's `MAX_CLUSTER_COUNT_LIMIT`.
const MAX_CLUSTER: usize = 64;

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

/// Load the connected component into `graph` in topological order, parent
/// edges remapped to local indices, and return the members: `graph`'s
/// local index `i` is `members[i]`. Callers must filter singletons before
/// calling - the singleton's `chunk_rate` is `fee/vsize`, set elsewhere.
pub fn load(txs: &[SnapTx], component: &[TxIndex], graph: &mut Cluster) -> Vec<TxIndex> {
    let position: FxHashMap<TxIndex, usize> = component
        .iter()
        .enumerate()
        .map(|(i, &idx)| (idx, i))
        .collect();
    let order = topo_sort(txs, component, &position);
    let mut local = vec![0u32; component.len()];
    for (i, &p) in order.iter().enumerate() {
        local[p] = i as u32;
    }
    graph.clear();
    for &p in &order {
        let t = &txs[component[p].as_usize()];
        graph.push(
            t.fee,
            t.vsize,
            t.parents
                .iter()
                .filter_map(|parent| position.get(parent))
                .map(|&q| CpfpClusterTxIndex::from(local[q])),
        );
    }
    order.into_iter().map(|p| component[p]).collect()
}

/// Kahn's topological sort over the connected component, restricted to
/// in-cluster parent edges. Returns component positions in an order where
/// every tx follows all its in-cluster parents.
fn topo_sort(
    txs: &[SnapTx],
    component: &[TxIndex],
    position: &FxHashMap<TxIndex, usize>,
) -> Vec<usize> {
    let n = component.len();
    let mut indeg: Vec<u32> = vec![0; n];
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &idx) in component.iter().enumerate() {
        let Some(t) = txs.get(idx.as_usize()) else {
            continue;
        };
        indeg[i] = t.parents.iter().filter(|p| position.contains_key(p)).count() as u32;
        for &c in t.children.iter() {
            if let Some(&ci) = position.get(&c) {
                children[i].push(ci);
            }
        }
    }
    let mut queue: VecDeque<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    let mut out: Vec<usize> = Vec::with_capacity(n);
    while let Some(i) = queue.pop_front() {
        out.push(i);
        for &c in &children[i] {
            indeg[c] -= 1;
            if indeg[c] == 0 {
                queue.push_back(c);
            }
        }
    }
    out
}
