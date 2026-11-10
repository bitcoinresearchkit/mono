//! Core's PostLinearize (`cluster_linearize.h`): passes over a linearization that never make it
//! worse. One pass each way makes its chunks connected, and the linearization optimal for trees
//! (every transaction with at most one parent, or at most one child).

use brk_types::{CpfpClusterTxIndex, Weight};

use super::compare_feerates;

use crate::cluster::Node;

/// No previous transaction in a group: its first.
const NO_PREVIOUS: usize = usize::MAX;

/// Core's PostLinearize state (`cluster_linearize.h`), entry 0 the list's sentinel: groups in a
/// list, each a run of the order, linked from its last transaction back to its first.
#[derive(Default)]
pub(super) struct Post {
    previous_tx: Vec<usize>,
    first_tx: Vec<usize>,
    previous_group: Vec<usize>,
    fee: Vec<i128>,
    weight: Vec<u64>,
    /// Each group's members and what it depends on (rows of `words`).
    members: Vec<u64>,
    deps: Vec<u64>,
}

impl Post {
    /// One pass over `order` (`rev`: back to front, parents and children swapped, feerates
    /// negated): each transaction joins the list as its own group, then moves ahead of every
    /// lower-feerate group before it, merging with one it depends on and swapping past one it
    /// doesn't. The order it leaves is never worse; one pass makes chunks connected, a back to
    /// front one is optimal when each transaction has at most one parent, a front to back one
    /// when each has at most one child.
    pub(super) fn pass(
        &mut self,
        nodes: &[Node],
        deps: &[u64],
        words: usize,
        order: &mut [CpfpClusterTxIndex],
        rev: bool,
    ) {
        let n = order.len();
        self.previous_tx.clear();
        self.previous_tx.resize(n + 1, NO_PREVIOUS);
        self.first_tx.clear();
        self.first_tx.resize(n + 1, 0);
        self.previous_group.clear();
        self.previous_group.resize(n + 1, 0);
        // The sentinel pays nothing over nothing: no group is ever above it.
        self.fee.clear();
        self.fee.resize(n + 1, 0);
        self.weight.clear();
        self.weight.resize(n + 1, 0);
        self.members.clear();
        self.members.resize((n + 1) * words, 0);
        self.deps.clear();
        self.deps.resize((n + 1) * words, 0);
        let above = |a: (i128, u64), b: (i128, u64)| {
            b.1 != 0 && compare_feerates((a.0, Weight::from(a.1)), (b.0, Weight::from(b.1))).is_gt()
        };
        for i in 0..n {
            let tx = u32::from(order[if rev { n - 1 - i } else { i }]) as usize;
            let current = tx + 1;
            self.members[current * words + tx / 64] |= 1u64 << (tx % 64);
            self.deps[current * words..][..words].copy_from_slice(&deps[tx * words..][..words]);
            let modified = i128::from(nodes[tx].modified);
            self.fee[current] = if rev { -modified } else { modified };
            self.weight[current] = u64::from(nodes[tx].weight);
            self.first_tx[current] = current;
            self.previous_group[current] = self.previous_group[0];
            self.previous_group[0] = current;
            let mut next = 0;
            let mut previous = self.previous_group[current];
            while above(
                (self.fee[current], self.weight[current]),
                (self.fee[previous], self.weight[previous]),
            ) {
                let depends = (0..words).any(|word| {
                    self.deps[current * words + word] & self.members[previous * words + word] != 0
                });
                if depends {
                    for word in 0..words {
                        self.members[current * words + word] |=
                            self.members[previous * words + word];
                        self.deps[current * words + word] |= self.deps[previous * words + word];
                    }
                    self.fee[current] += self.fee[previous];
                    self.weight[current] += self.weight[previous];
                    let first = self.first_tx[current];
                    self.previous_tx[first] = previous;
                    self.first_tx[current] = self.first_tx[previous];
                    previous = self.previous_group[previous];
                    self.previous_group[current] = previous;
                } else {
                    let before = self.previous_group[previous];
                    self.previous_group[next] = previous;
                    self.previous_group[previous] = current;
                    self.previous_group[current] = before;
                    next = previous;
                    previous = before;
                }
            }
        }
        // From the last group back, each from its last transaction back: written from the end, or
        // (a back to front pass's list being reversed) from the start.
        let mut group = self.previous_group[0];
        let mut written = 0;
        while group != 0 {
            let mut tx = group;
            while tx != NO_PREVIOUS {
                let index = CpfpClusterTxIndex::from((tx - 1) as u32);
                if rev {
                    order[written] = index;
                } else {
                    order[n - 1 - written] = index;
                }
                written += 1;
                tx = self.previous_tx[tx];
            }
            group = self.previous_group[group];
        }
        debug_assert_eq!(written, n, "every transaction written once");
    }
}
