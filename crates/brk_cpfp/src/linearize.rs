//! Ancestor-set linearization and chunking for topologically ordered dependency components.
//!
//! Repeatedly take the remaining transaction whose remaining ancestor set pays the most per
//! vbyte (ties: the larger set, then the earlier transaction), then chunk that order: a group
//! paying more than the chunk before it merges into it, so chunk feerates never increase and
//! every chunk follows the ones it depends on. Feerates compare as exact fractions; only the
//! published chunk feerate rounds.
//!
//! Ancestor sets are immutable bitsets, one row per transaction in a single flat buffer;
//! taking a set subtracts its fee and size from every remaining candidate that contains its
//! members. Selection and updates take O(n²) work; building the bitsets takes
//! O((n + edges) * ceil(n / 64)) work and O(n² / 64) words of scratch space. This also
//! supports confirmed components larger than mempool policy limits.

use std::cmp::Ordering;

use brk_types::{CpfpClusterTxIndex, FeeRate, Sats, VSize};

use crate::cluster::Node;

/// Buffers reused across linearizations.
#[derive(Default)]
pub(crate) struct Scratch {
    /// Row `i` (`ceil(n / 64)` words) is transaction `i`'s ancestor set, itself included.
    ancestors: Vec<u64>,
    /// Each candidate's remaining ancestor set totals.
    sets: Vec<Totals>,
    remaining: Vec<bool>,
    /// Transactions in selection order: every group, and so every chunk, is a contiguous run.
    order: Vec<CpfpClusterTxIndex>,
    chunks: Vec<Chunk>,
}

struct Totals {
    fee: Sats,
    vsize: VSize,
    count: usize,
}

/// A chunk: `order[previous end..end]`, with its summed fee and size.
struct Chunk {
    end: usize,
    fee: Sats,
    vsize: VSize,
}

/// `a`'s feerate against `b`'s, exactly: fee_a / vsize_a vs fee_b / vsize_b.
fn compare_rates(a: (Sats, VSize), b: (Sats, VSize)) -> Ordering {
    let (fee_a, vsize_a) = (u128::from(u64::from(a.0)), u128::from(u64::from(a.1)));
    let (fee_b, vsize_b) = (u128::from(u64::from(b.0)), u128::from(u64::from(b.1)));
    (fee_a * vsize_b).cmp(&(fee_b * vsize_a))
}

fn contains(row: &[u64], index: usize) -> bool {
    row[index / 64] & (1u64 << (index % 64)) != 0
}

/// The set bits of `row`, ascending.
fn members(row: &[u64]) -> impl Iterator<Item = usize> + '_ {
    row.iter().enumerate().flat_map(|(word_index, &word)| {
        let mut bits = word;
        std::iter::from_fn(move || {
            (bits != 0).then(|| {
                let bit = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                word_index * 64 + bit
            })
        })
    })
}

pub(crate) fn linearize(
    nodes: &[Node],
    parents: &[CpfpClusterTxIndex],
    scratch: &mut Scratch,
    mut emit: impl FnMut(&[CpfpClusterTxIndex], FeeRate),
) {
    let n = nodes.len();
    let words = n.div_ceil(64);
    let Scratch {
        ancestors,
        sets,
        remaining,
        order,
        chunks,
    } = scratch;
    ancestors.clear();
    ancestors.resize(n * words, 0);
    sets.clear();
    remaining.clear();
    remaining.resize(n, true);
    order.clear();
    chunks.clear();

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
            fee: Sats::ZERO,
            vsize: VSize::from(0u64),
            count: 0,
        };
        for ancestor in members(row) {
            totals.fee += nodes[ancestor].fee;
            totals.vsize += nodes[ancestor].vsize;
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
                compare_rates((set.fee, set.vsize), (best.fee, best.vsize))
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
                    set.fee -= nodes[removed].fee;
                    set.vsize -= nodes[removed].vsize;
                    set.count -= 1;
                }
            }
        }

        // A group paying more than the chunk before it joins that chunk.
        let mut chunk = Chunk {
            end: order.len(),
            fee: sets[selected].fee,
            vsize: sets[selected].vsize,
        };
        while let Some(last) = chunks.last()
            && compare_rates((chunk.fee, chunk.vsize), (last.fee, last.vsize)).is_gt()
        {
            chunk.fee += last.fee;
            chunk.vsize += last.vsize;
            chunks.pop();
        }
        chunks.push(chunk);
    }

    let mut start = 0;
    for chunk in chunks.iter() {
        let txs = &mut order[start..chunk.end];
        txs.sort_unstable_by_key(|index| u32::from(*index));
        emit(txs, FeeRate::from((chunk.fee, chunk.vsize)));
        start = chunk.end;
    }
}
