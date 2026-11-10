//! The optimal linearization of a component of at most 64 members.
//!
//! It repeatedly takes, among the remaining members, the closure (a set holding its own remaining
//! ancestors) that pays the most per weight unit. That sequence is optimal, and each group it
//! takes is a chunk. The best closure is found exactly by Dinkelbach's method: from the best
//! ancestor set's feerate, look for a closure doing better at that feerate (a maximum-weight
//! closure, [`super::flow`]); while one exists, its feerate becomes the bar. When none does, the
//! smallest closure paying exactly as much is the group, so the chunks come out minimal, as Core's.
//!
//! The chunks are then ordered as Core's `GetLinearization` orders them, and Core's
//! PostLinearize runs over the result, as Core runs it after every linearization.

use brk_types::{CpfpClusterTxIndex, FeeRate, Weight};

use super::{EXACT_MAX, Scratch, compare_rates, emit_chunks, push_group};
use crate::cluster::Node;

/// A set of a component's members, bit `i` member `i`.
pub(super) type Set = u64;

pub(super) fn bit(index: usize) -> Set {
    1 << index
}

pub(super) fn set_members(set: Set) -> impl Iterator<Item = usize> {
    let mut bits = set;
    std::iter::from_fn(move || {
        (bits != 0).then(|| {
            let index = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            index
        })
    })
}

fn totals(nodes: &[Node], set: Set) -> (i128, Weight) {
    set_members(set).fold((0, Weight::default()), |(fee, weight), index| {
        (fee + i128::from(nodes[index].modified), weight + nodes[index].weight)
    })
}

pub(super) fn exact(
    nodes: &[Node],
    parents: &[CpfpClusterTxIndex],
    scratch: &mut Scratch,
    emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let n = nodes.len();
    let mut direct = [0 as Set; EXACT_MAX];
    let mut ancestors = [0 as Set; EXACT_MAX];
    for (index, node) in nodes.iter().enumerate() {
        ancestors[index] = bit(index);
        for &parent in &parents[node.parents.clone()] {
            let parent = u32::from(parent) as usize;
            direct[index] |= bit(parent);
            ancestors[index] |= ancestors[parent];
        }
    }
    let mut remaining: Set = if n == EXACT_MAX { Set::MAX } else { bit(n) - 1 };
    let mut groups: Vec<Set> = Vec::with_capacity(n);
    while remaining != 0 {
        // From the best remaining ancestor set (more members, then the earliest, on ties).
        let mut best = set_members(remaining)
            .map(|index| ancestors[index] & remaining)
            .max_by(|&a, &b| {
                compare_rates(totals(nodes, a), totals(nodes, b))
                    .then(a.count_ones().cmp(&b.count_ones()))
                    .then(b.trailing_zeros().cmp(&a.trailing_zeros()))
            })
            .expect("a remaining member");
        // Dinkelbach: a closure paying more than `best` raises the bar, until none does. Then
        // the smallest closure paying exactly as much is the group.
        loop {
            let (fee, weight) = totals(nodes, best);
            let (p, q) = (fee, i128::from(u64::from(weight)));
            let weight = |index: usize| {
                i128::from(nodes[index].modified) * q
                    - i128::from(u64::from(nodes[index].weight)) * p
            };
            let (closure, value) = scratch.flow.max_closure(remaining, &direct, weight);
            if value > 0 {
                best = closure;
                continue;
            }
            // At the best feerate every best closure weighs zero: the smallest nonempty one (a
            // lone member is one), read off this last cut.
            if best.count_ones() > 1 {
                best = scratch.flow.smallest_tight_closure(remaining, &ancestors, best);
            }
            break;
        }
        groups.push(best);
        remaining &= !best;
    }
    core_order(nodes, &direct, &ancestors, groups, &mut scratch.order);
    // Then Core's PostLinearize, as Core runs it after every linearization (on an optimal one it
    // moves only the order inside chunks), and the result chunked.
    let Scratch {
        ancestors: rows,
        descendants,
        post,
        order,
        chunks,
        ..
    } = scratch;
    rows.clear();
    rows.extend_from_slice(&ancestors[..n]);
    descendants.clear();
    descendants.resize(n, 0);
    for (index, &set) in ancestors[..n].iter().enumerate() {
        for ancestor in set_members(set) {
            descendants[ancestor] |= bit(index);
        }
    }
    post.pass(nodes, descendants, 1, order, true);
    post.pass(nodes, rows, 1, order, false);
    chunks.clear();
    for (end, &index) in order.iter().enumerate() {
        let node = &nodes[u32::from(index) as usize];
        push_group(chunks, end + 1, node.modified, node.weight);
    }
    emit_chunks(order, chunks, emit);
}

/// The optimal chunks in Core's order (`GetLinearization`), into `order`: of the chunks whose
/// ancestors are out, the highest feerate, then the smallest weight, then the lowest greatest
/// fallback place; inside a chunk, of the members whose parents are out, the highest own feerate,
/// then the smallest weight, then the lowest fallback place. Only equal feerates move: the chunks
/// come found by non-increasing feerate, each after its ancestors.
fn core_order(
    nodes: &[Node],
    direct: &[Set],
    ancestors: &[Set],
    mut groups: Vec<Set>,
    order: &mut Vec<CpfpClusterTxIndex>,
) {
    let weight = |set: Set| totals(nodes, set).1;
    let last = |set: Set| set_members(set).map(|index| nodes[index].fallback).max();
    let mut out: Set = 0;
    order.clear();
    while !groups.is_empty() {
        let ready = |group: &Set| {
            set_members(*group).all(|member| ancestors[member] & !group & !out == 0)
        };
        let next = (0..groups.len())
            .filter(|&i| ready(&groups[i]))
            .max_by(|&a, &b| {
                let (a, b) = (groups[a], groups[b]);
                compare_rates(totals(nodes, a), totals(nodes, b))
                    .then(weight(b).cmp(&weight(a)))
                    .then(last(b).cmp(&last(a)))
            })
            .expect("a chunk whose ancestors are out");
        let group = groups.remove(next);
        let mut left = group;
        while left != 0 {
            let member = set_members(left)
                .filter(|&member| direct[member] & left == 0)
                .max_by(|&a, &b| {
                    let (a, b) = (&nodes[a], &nodes[b]);
                    compare_rates((a.modified, a.weight), (b.modified, b.weight))
                        .then(b.weight.cmp(&a.weight))
                        .then(b.fallback.cmp(&a.fallback))
                })
                .expect("a member whose parents are out");
            order.push(CpfpClusterTxIndex::from(member as u32));
            left &= !bit(member);
        }
        out |= group;
    }
}
