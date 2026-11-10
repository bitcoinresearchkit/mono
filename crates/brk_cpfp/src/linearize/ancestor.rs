//! The ancestor-set heuristic, for components up to 4,096 members.
//!
//! It repeatedly takes the remaining ancestor set paying the most per weight unit (ties: the
//! larger set, then the earlier transaction), runs Core's PostLinearize over that order, and
//! chunks the result. Ancestor sets are bitset rows in one flat buffer. Taking a set subtracts
//! its fee and weight from every remaining set that contains its members: O(n²) work, and about
//! n²/2 bytes with the descendant rows.

use brk_types::{CpfpClusterTxIndex, FeeRate, Weight};

use super::{Scratch, Totals, compare_rates, contains, emit_chunks, members, push_group};
use crate::cluster::Node;

/// The best remaining ancestor sets, then Core's PostLinearize over that order, chunked.
pub(crate) fn ancestor_sets(
    nodes: &[Node],
    parents: &[CpfpClusterTxIndex],
    scratch: &mut Scratch,
    emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    greedy(nodes, parents, scratch);
    let words = nodes.len().div_ceil(64);
    let Scratch {
        ancestors,
        descendants,
        post,
        order,
        chunks,
        ..
    } = scratch;
    // Each transaction's descendants, itself included: the ancestor rows transposed.
    descendants.clear();
    descendants.resize(ancestors.len(), 0);
    for index in 0..nodes.len() {
        for ancestor in members(&ancestors[index * words..][..words]) {
            descendants[ancestor * words + index / 64] |= 1u64 << (index % 64);
        }
    }
    // Back to front over descendants, then front to back over ancestors, as Core.
    post.pass(nodes, descendants, words, order, true);
    post.pass(nodes, ancestors, words, order, false);
    chunks.clear();
    for (end, &index) in order.iter().enumerate() {
        let node = &nodes[u32::from(index) as usize];
        push_group(chunks, end + 1, node.modified, node.weight);
    }
    emit_chunks(order, chunks, emit);
}

/// The best remaining ancestor set, repeatedly (ties: the larger set, then the earlier
/// transaction), into `scratch.order`.
fn greedy(nodes: &[Node], parents: &[CpfpClusterTxIndex], scratch: &mut Scratch) {
    let n = nodes.len();
    let words = n.div_ceil(64);
    let Scratch {
        ancestors,
        sets,
        remaining,
        order,
        ..
    } = scratch;
    ancestors.clear();
    ancestors.resize(n * words, 0);
    sets.clear();
    remaining.clear();
    remaining.resize(n, true);
    order.clear();

    for (index, node) in nodes.iter().enumerate() {
        let (earlier, row) = ancestors.split_at_mut(index * words);
        let row = &mut row[..words];
        row[index / 64] |= 1u64 << (index % 64);
        for &parent in &parents[node.parents.clone()] {
            let parent = u32::from(parent) as usize;
            for (word, parent_word) in row.iter_mut().zip(&earlier[parent * words..][..words]) {
                *word |= parent_word;
            }
        }
        let mut totals = Totals {
            fee: 0,
            weight: Weight::default(),
            count: 0,
        };
        for ancestor in members(row) {
            totals.fee += i128::from(nodes[ancestor].modified);
            totals.weight += nodes[ancestor].weight;
            totals.count += 1;
        }
        sets.push(totals);
    }

    loop {
        let mut best: Option<usize> = None;
        for (index, set) in sets.iter().enumerate() {
            if !remaining[index] {
                continue;
            }
            if best.is_none_or(|best| {
                let best = &sets[best];
                compare_rates((set.fee, set.weight), (best.fee, best.weight))
                    .then(set.count.cmp(&best.count))
                    .is_gt()
            }) {
                best = Some(index);
            }
        }
        let Some(selected) = best else {
            break;
        };

        let group_start = order.len();
        for index in members(&ancestors[selected * words..][..words]) {
            if remaining[index] {
                remaining[index] = false;
                order.push(CpfpClusterTxIndex::from(index as u32));
            }
        }
        let group = &order[group_start..];

        // Each transaction is subtracted at most once from each candidate.
        for (index, set) in sets.iter_mut().enumerate() {
            if !remaining[index] {
                continue;
            }
            let row = &ancestors[index * words..][..words];
            for &removed in group {
                let removed = u32::from(removed) as usize;
                if contains(row, removed) {
                    set.fee -= i128::from(nodes[removed].modified);
                    set.weight -= nodes[removed].weight;
                    set.count -= 1;
                }
            }
        }
    }
}
