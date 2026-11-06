//! Cluster primitives over `SnapTx` adjacency: connected-component
//! discovery, topo-sort, and loading into a [`brk_cpfp::Cluster`] (shared
//! with bitview_query's confirmed CPFP and the transactions plugin).

use std::{cmp::Reverse, collections::BinaryHeap};

use brk_cpfp::Cluster;
use brk_types::{CpfpClusterTxIndex, FeeRate, Sats, Weight};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{SnapTx, TxIndex};

/// The whole connected component holding `seed` over `parents ∪ children`, in txid order: one
/// order whichever member a walk starts from and however the pool's slots fall, so every
/// linearization of it (the snapshot's rates, each CPFP answer, a cached one) is the same. Never
/// cut short: Core 31 keeps a cluster to 64 transactions; an older or other node's can be larger,
/// linearized whole.
pub fn walk(txs: &[SnapTx], seed: TxIndex) -> Vec<TxIndex> {
    debug_assert!(seed.as_usize() < txs.len());
    let mut visited: FxHashSet<TxIndex> = FxHashSet::default();
    visited.insert(seed);
    let mut out = vec![seed];
    let mut stack = vec![seed];
    while let Some(idx) = stack.pop() {
        let t = &txs[idx.as_usize()];
        for &n in t.parents.iter().chain(t.children.iter()) {
            if visited.insert(n) {
                out.push(n);
                stack.push(n);
            }
        }
    }
    out.sort_unstable_by(|&a, &b| txs[a.as_usize()].txid.cmp(&txs[b.as_usize()].txid));
    out
}

/// Load the connected component into `graph` in topological order, parent
/// edges remapped to local indices, and return the members: `graph`'s
/// local index `i` is `members[i]`.
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
        graph.push_ranked(
            t.fee,
            t.modified,
            t.sigops.adjust_weight(t.weight),
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
/// every tx follows all its in-cluster parents, the earliest ready position first: the order
/// depends on the component's own order alone.
fn topo_sort(
    txs: &[SnapTx],
    component: &[TxIndex],
    position: &FxHashMap<TxIndex, usize>,
) -> Vec<usize> {
    let n = component.len();
    let mut indeg: Vec<u32> = vec![0; n];
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, &idx) in component.iter().enumerate() {
        let t = &txs[idx.as_usize()];
        indeg[i] = t
            .parents
            .iter()
            .filter(|p| position.contains_key(p))
            .count() as u32;
        for &c in t.children.iter() {
            if let Some(&ci) = position.get(&c) {
                children[i].push(ci);
            }
        }
    }
    let mut ready: BinaryHeap<Reverse<usize>> =
        (0..n).filter(|&i| indeg[i] == 0).map(Reverse).collect();
    let mut out: Vec<usize> = Vec::with_capacity(n);
    while let Some(Reverse(i)) = ready.pop() {
        out.push(i);
        for &c in &children[i] {
            indeg[c] -= 1;
            if indeg[c] == 0 {
                ready.push(Reverse(c));
            }
        }
    }
    out
}

/// A chunk's feerate by what its members pay (base fees) over their policy weight: what is shown.
/// Linearization and packing rank by modified fees instead; without an operator's delta the two
/// agree.
pub fn paid_rate(txs: &[SnapTx], members: impl IntoIterator<Item = TxIndex>) -> FeeRate {
    let (fee, weight) = members
        .into_iter()
        .map(|m| &txs[m.as_usize()])
        .fold((Sats::ZERO, Weight::default()), |(fee, weight), t| {
            (fee + t.fee, weight + t.sigops.adjust_weight(t.weight))
        });
    FeeRate::from((fee, weight))
}
