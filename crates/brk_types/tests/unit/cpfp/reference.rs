// Slow pre-optimization implementation, retained only as a parity oracle.

use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::{ChunkInput, CpfpClusterChunk, CpfpClusterTxIndex, FeeRate, Sats, VSize};

/// Linearize `items` into chunks. `items` must be in topological order
/// (parents before children); `parents` indices must point earlier in
/// the slice. Returns chunks sorted by descending feerate, with each
/// chunk's `txs` listed in the input topological order.
pub(crate) fn linearize(items: &[ChunkInput<'_>]) -> Vec<CpfpClusterChunk> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let mut remaining: Vec<bool> = vec![true; n];
    let mut chunks: Vec<CpfpClusterChunk> = Vec::new();

    while remaining.iter().any(|&r| r) {
        // Build one chunk by repeatedly absorbing the highest-rate
        // ancestor-closed extension. Starting from empty: the first
        // pick is the top-rate single-anchor closure (Phase 1 of
        // canonical SFL); subsequent picks merge in disjoint
        // components whose rate is >= the chunk's current rate, which
        // is what makes a parent + chain + same-rate siblings collapse
        // into one chunk instead of trailing same-rate singletons that
        // can even sort "above" the main chunk under integer-vsize
        // rounding. Size tiebreak ensures a uniform-rate chain is
        // taken in one swallow.
        let mut anc: FxHashSet<u32> = FxHashSet::default();
        let mut chunk_fee = Sats::ZERO;
        let mut chunk_vsize = VSize::from(0u64);
        let mut chunk_rate: Option<FeeRate> = None;

        loop {
            let mut best: Option<(FeeRate, FxHashSet<u32>, Sats, VSize)> = None;
            for i in 0..n {
                if !remaining[i] || anc.contains(&(i as u32)) {
                    continue;
                }
                let extra = closure(items, &remaining, &anc, i as u32);
                if extra.is_empty() {
                    continue;
                }
                let (ef, ev) = sum_fee_vsize(items, &extra);
                let new_fee = chunk_fee + ef;
                let new_vsize = chunk_vsize + ev;
                let new_rate = FeeRate::from((new_fee, new_vsize));
                if chunk_rate.is_some_and(|cr| new_rate < cr) {
                    continue;
                }
                let replace = match &best {
                    None => true,
                    Some((br, ba, _, _)) => {
                        new_rate > *br || (new_rate == *br && extra.len() > ba.len())
                    }
                };
                if replace {
                    best = Some((new_rate, extra, new_fee, new_vsize));
                }
            }
            match best {
                Some((r, e, f, v)) => {
                    anc.extend(&e);
                    chunk_fee = f;
                    chunk_vsize = v;
                    chunk_rate = Some(r);
                }
                None => break,
            }
        }

        let mut indices: Vec<u32> = anc.into_iter().collect();
        indices.sort_unstable();
        for &x in &indices {
            remaining[x as usize] = false;
        }
        let txs: Vec<CpfpClusterTxIndex> =
            indices.into_iter().map(CpfpClusterTxIndex::from).collect();
        chunks.push(CpfpClusterChunk {
            txs,
            feerate: chunk_rate.expect("at least one remaining tx"),
        });
    }
    chunks
}

fn closure(
    items: &[ChunkInput<'_>],
    remaining: &[bool],
    excluded: &FxHashSet<u32>,
    start: u32,
) -> FxHashSet<u32> {
    let mut set: FxHashSet<u32> = FxHashSet::with_capacity_and_hasher(8, FxBuildHasher);
    if !remaining[start as usize] || excluded.contains(&start) {
        return set;
    }
    let mut stack: Vec<u32> = vec![start];
    while let Some(x) = stack.pop() {
        if !set.insert(x) {
            continue;
        }
        for &p in items[x as usize].parents {
            let pu: u32 = u32::from(p);
            if remaining[pu as usize] && !excluded.contains(&pu) && !set.contains(&pu) {
                stack.push(pu);
            }
        }
    }
    set
}

fn sum_fee_vsize(items: &[ChunkInput<'_>], set: &FxHashSet<u32>) -> (Sats, VSize) {
    let mut fee = Sats::ZERO;
    let mut vsize = VSize::from(0u64);
    for &x in set {
        fee += items[x as usize].fee;
        vsize += items[x as usize].vsize;
    }
    (fee, vsize)
}
