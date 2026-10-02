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

#[cfg(test)]
mod tests {
    use bitcoin::{Txid as BitcoinTxid, hashes::Hash};
    use brk_types::{Sats, Txid, Weight};
    use smallvec::SmallVec;

    use super::*;

    fn snap_tx(seed: u8, fee: u64, vsize: u64) -> SnapTx {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        SnapTx {
            txid: Txid::from(BitcoinTxid::from_byte_array(bytes)),
            fee: Sats::from(fee),
            vsize: VSize::from(vsize),
            weight: Weight::from(vsize * 4),
            size: vsize,
            chunk_rate: FeeRate::from((Sats::from(fee), VSize::from(vsize))),
            parents: SmallVec::new(),
            children: SmallVec::new(),
        }
    }

    #[test]
    fn packing_preserves_rate_ties_exclusions_and_last_block_overflow() {
        let big = u64::from(VSize::MAX_BLOCK) - 100;
        let txs = [
            snap_tx(0x20, big * 3, big),
            snap_tx(0x00, big * 100, big),
            snap_tx(0x10, big * 3, big),
            snap_tx(0x30, big * 2, big),
        ];
        for (slots, expected) in [
            (1, vec![vec![2, 0, 3]]),
            (2, vec![vec![2], vec![0, 3]]),
            (3, vec![vec![2], vec![0], vec![3]]),
        ] {
            let actual: Vec<Vec<_>> = partition(&txs, &[0, 1, 0, 0], slots)
                .into_iter()
                .map(|block| block.into_iter().map(|index| index.as_usize()).collect())
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
