//! Single Fee Linearization for topologically ordered dependency components.
//!
//! Preserve the greedy ancestor-closed extension and tie-breaking rules without
//! rebuilding every candidate's ancestor closure for each chunk. Ancestor sets
//! are immutable bitsets; accepting a transaction subtracts its fee and size
//! from every remaining candidate that depends on it.
//!
//! Selection and aggregate updates take O(n²) work. Building ancestor bitsets
//! takes O((n + edges) * ceil(n / 64)) work and O(n² / 64) words of scratch space.
//! This also supports confirmed components larger than mempool policy limits.

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
    let mut remaining_count = n;
    let mut chunks = Vec::new();
    while remaining_count != 0 {
        let mut txs = Vec::new();
        let mut fee = Sats::ZERO;
        let mut vsize = VSize::from(0u64);
        let mut rate: Option<FeeRate> = None;

        loop {
            let mut best: Option<(usize, FeeRate)> = None;
            for (index, candidate) in candidates.iter().enumerate() {
                if !remaining[index] {
                    continue;
                }
                let next_rate = FeeRate::from((fee + candidate.fee, vsize + candidate.vsize));
                if rate.is_some_and(|rate| next_rate < rate) {
                    continue;
                }
                if best.is_none_or(|(best_index, best_rate)| {
                    next_rate > best_rate
                        || (next_rate == best_rate
                            && candidate.count > candidates[best_index].count)
                }) {
                    best = Some((index, next_rate));
                }
            }

            let Some((selected, next_rate)) = best else {
                break;
            };
            fee += candidates[selected].fee;
            vsize += candidates[selected].vsize;
            rate = Some(next_rate);

            let added: Vec<_> = (0..n)
                .filter(|&index| remaining[index] && candidates[selected].contains(index))
                .collect();
            for &index in &added {
                remaining[index] = false;
                txs.push(CpfpClusterTxIndex::from(index as u32));
            }
            remaining_count -= added.len();

            // Each transaction is subtracted at most once from each candidate.
            // Completed chunks and the current chunk share the same exclusion.
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
        }

        txs.sort_unstable_by_key(|index| u32::from(*index));
        chunks.push(CpfpClusterChunk {
            txs,
            feerate: rate.expect("a remaining transaction forms a chunk"),
        });
    }
    chunks
}
