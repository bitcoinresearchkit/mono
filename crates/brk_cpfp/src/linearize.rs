//! Ancestor-set linearization and chunking for topologically ordered dependency components.
//!
//! Repeatedly take the remaining transaction whose remaining ancestor set pays the most per
//! vbyte (ties: the larger set, then the earlier transaction), then chunk that order: a group
//! paying more than the chunk before it merges into it, so chunk feerates never increase and
//! every chunk follows the ones it depends on. Feerates compare as exact fractions; only the
//! published chunk feerate rounds.
//!
//! Ancestor sets are immutable bitsets; taking a set subtracts its fee and size from every
//! remaining candidate that contains its members. Selection and updates take O(n²) work;
//! building the bitsets takes O((n + edges) * ceil(n / 64)) work and O(n² / 64) words of
//! scratch space. This also supports confirmed components larger than mempool policy limits.

use std::cmp::Ordering;

use brk_types::{CpfpClusterChunk, CpfpClusterTxIndex, FeeRate, Sats, VSize};

use crate::ChunkInput;

struct Candidate {
    ancestors: Vec<u64>,
    fee: Sats,
    vsize: VSize,
    count: usize,
}

impl Candidate {
    fn contains(&self, index: usize) -> bool {
        self.ancestors[index / 64] & (1u64 << (index % 64)) != 0
    }
}

/// Transactions with their summed fee and size.
struct Group {
    txs: Vec<CpfpClusterTxIndex>,
    fee: Sats,
    vsize: VSize,
}

/// `a`'s feerate against `b`'s, exactly: fee_a / vsize_a vs fee_b / vsize_b.
fn compare_rates(a: (Sats, VSize), b: (Sats, VSize)) -> Ordering {
    let (fee_a, vsize_a) = (u128::from(u64::from(a.0)), u128::from(u64::from(a.1)));
    let (fee_b, vsize_b) = (u128::from(u64::from(b.0)), u128::from(u64::from(b.1)));
    (fee_a * vsize_b).cmp(&(fee_b * vsize_a))
}

/// Linearize `items` into descending-feerate chunks, preserving input order
/// within each chunk. Parents must point earlier in the topological input.
pub fn linearize(items: &[ChunkInput<'_>]) -> Vec<CpfpClusterChunk> {
    let n = items.len();
    let mut candidates: Vec<Candidate> = Vec::with_capacity(n);
    for (index, item) in items.iter().enumerate() {
        let mut ancestors = vec![0u64; n.div_ceil(64)];
        ancestors[index / 64] |= 1u64 << (index % 64);
        for &parent in item.parents {
            let parent = u32::from(parent) as usize;
            assert!(parent < index, "CPFP parents must precede their children");
            for (word, parent_word) in ancestors.iter_mut().zip(&candidates[parent].ancestors) {
                *word |= parent_word;
            }
        }
        let mut candidate = Candidate {
            ancestors,
            fee: Sats::ZERO,
            vsize: VSize::from(0u64),
            count: 0,
        };
        for (ancestor, input) in items[..=index].iter().enumerate() {
            if candidate.contains(ancestor) {
                candidate.fee += input.fee;
                candidate.vsize += input.vsize;
                candidate.count += 1;
            }
        }
        candidates.push(candidate);
    }

    let mut remaining = vec![true; n];
    let mut chunks: Vec<Group> = Vec::new();
    loop {
        let mut best: Option<usize> = None;
        for (index, candidate) in candidates.iter().enumerate() {
            if !remaining[index] {
                continue;
            }
            if best.is_none_or(|best| {
                let best = &candidates[best];
                compare_rates((candidate.fee, candidate.vsize), (best.fee, best.vsize))
                    .then(candidate.count.cmp(&best.count))
                    .is_gt()
            }) {
                best = Some(index);
            }
        }
        let Some(selected) = best else {
            break;
        };

        let added: Vec<usize> = (0..n)
            .filter(|&index| remaining[index] && candidates[selected].contains(index))
            .collect();
        let mut group = Group {
            txs: added
                .iter()
                .map(|&index| CpfpClusterTxIndex::from(index as u32))
                .collect(),
            fee: candidates[selected].fee,
            vsize: candidates[selected].vsize,
        };
        for &index in &added {
            remaining[index] = false;
        }

        // Each transaction is subtracted at most once from each candidate.
        for (index, candidate) in candidates.iter_mut().enumerate() {
            if !remaining[index] {
                continue;
            }
            for &removed in &added {
                if candidate.contains(removed) {
                    candidate.fee -= items[removed].fee;
                    candidate.vsize -= items[removed].vsize;
                    candidate.count -= 1;
                }
            }
        }

        // A group paying more than the chunk before it joins that chunk.
        while let Some(last) = chunks.last()
            && compare_rates((group.fee, group.vsize), (last.fee, last.vsize)).is_gt()
        {
            let mut last = chunks.pop().expect("checked above");
            last.txs.append(&mut group.txs);
            last.fee += group.fee;
            last.vsize += group.vsize;
            group = last;
        }
        chunks.push(group);
    }

    chunks
        .into_iter()
        .map(|mut chunk| {
            chunk.txs.sort_unstable_by_key(|index| u32::from(*index));
            CpfpClusterChunk {
                feerate: FeeRate::from((chunk.fee, chunk.vsize)),
                txs: chunk.txs,
            }
        })
        .collect()
}
