//! Pack live txs into projected blocks 1..N by descending `chunk_rate`.
//! Block 0 is filled by the caller from `getblocktemplate`. Final block
//! is a catch-all (no vsize cap).

use brk_types::{FeeRate, VSize};

use super::{SnapTx, TxIndex};

pub fn partition(
    txs: &[SnapTx],
    excluded: &[u8],
    num_remaining_blocks: usize,
) -> Vec<Vec<TxIndex>> {
    if num_remaining_blocks == 0 {
        return Vec::new();
    }
    let sorted = sorted_candidates(txs, excluded);
    let mut blocks: Vec<Vec<TxIndex>> = (0..num_remaining_blocks).map(|_| Vec::new()).collect();
    let mut block_vsize = VSize::default();
    let mut current = 0;
    let last = num_remaining_blocks - 1;
    for (idx, vsize, _) in sorted {
        let fits = vsize <= VSize::MAX_BLOCK.saturating_sub(block_vsize);
        if !fits && current < last && !blocks[current].is_empty() {
            current += 1;
            block_vsize = VSize::default();
        }
        blocks[current].push(idx);
        block_vsize += vsize;
    }
    blocks
}

fn sorted_candidates(txs: &[SnapTx], excluded: &[u8]) -> Vec<(TxIndex, VSize, FeeRate)> {
    debug_assert_eq!(txs.len(), excluded.len());
    let mut cands: Vec<(TxIndex, VSize, FeeRate)> = txs
        .iter()
        .zip(excluded)
        .enumerate()
        .filter_map(|(i, (t, &excluded))| {
            let idx = TxIndex::from(i);
            (excluded == 0).then_some((idx, t.vsize, t.chunk_rate))
        })
        .collect();
    cands.sort_unstable_by(|(a_idx, _, a_rate), (b_idx, _, b_rate)| {
        b_rate
            .cmp(a_rate)
            .then_with(|| txs[a_idx.as_usize()].txid.cmp(&txs[b_idx.as_usize()].txid))
    });
    cands
}
