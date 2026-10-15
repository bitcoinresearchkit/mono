//! Pack live txs into projected blocks 1..N by descending `chunk_rate`.
//! Block 0 is filled by the caller from `getblocktemplate`. The final block
//! is a catch-all (no vsize cap); trailing empty blocks are not made.

use brk_types::{FeeRate, VSize};

use super::{SnapTx, TxIndex};

pub fn partition(
    txs: &[SnapTx],
    excluded: &[u8],
    num_remaining_blocks: usize,
) -> Vec<Vec<TxIndex>> {
    // Only blocks that hold transactions: fee tiers read a missing next block as the end of
    // the mempool, as mempool.space does.
    let mut blocks: Vec<Vec<TxIndex>> = Vec::new();
    if num_remaining_blocks == 0 {
        return blocks;
    }
    let mut block_vsize = VSize::default();
    for (idx, vsize, _) in sorted_candidates(txs, excluded) {
        let fits = vsize <= VSize::MAX_BLOCK.saturating_sub(block_vsize);
        if blocks.is_empty() || (!fits && blocks.len() < num_remaining_blocks) {
            blocks.push(Vec::new());
            block_vsize = VSize::default();
        }
        blocks.last_mut().unwrap().push(idx);
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
