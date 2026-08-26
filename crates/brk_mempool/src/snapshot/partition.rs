//! Pack live txs into projected blocks 1..N by descending `chunk_rate`.
//! Block 0 is filled by the caller from `getblocktemplate`. Final block
//! is a catch-all (no vsize cap).

use brk_types::{FeeRate, VSize};

use super::{SnapTx, TxIndex};

pub struct Partitioner;

impl Partitioner {
    pub fn partition(
        txs: &[SnapTx],
        excluded: &[u8],
        num_remaining_blocks: usize,
    ) -> Vec<Vec<TxIndex>> {
        if num_remaining_blocks == 0 {
            return Vec::new();
        }
        let sorted = Self::sorted_candidates(txs, excluded);
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
}

#[cfg(test)]
mod tests {
    use bitcoin::hashes::Hash;
    use brk_types::{Sats, Txid, Weight};
    use smallvec::SmallVec;

    use super::*;

    fn snap_tx(seed: u8, fee: u64, vsize: u64) -> SnapTx {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        SnapTx {
            txid: Txid::from(bitcoin::Txid::from_byte_array(bytes)),
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
    fn zero_blocks_returns_empty() {
        let txs = vec![snap_tx(1, 100, 100)];
        let blocks = Partitioner::partition(&txs, &[0], 0);
        assert!(blocks.is_empty());
    }

    #[test]
    fn higher_chunk_rate_packs_first() {
        let txs = vec![snap_tx(1, 100, 100), snap_tx(2, 1_000, 100)];
        let blocks = Partitioner::partition(&txs, &[0, 0], 3);
        assert_eq!(blocks[0][0], TxIndex::from(1usize));
        assert_eq!(blocks[0][1], TxIndex::from(0usize));
    }

    #[test]
    fn excluded_txs_are_skipped() {
        let txs = vec![snap_tx(1, 100, 100), snap_tx(2, 1_000, 100)];
        let excluded = [0, 1];
        let blocks = Partitioner::partition(&txs, &excluded, 3);
        let flat: Vec<TxIndex> = blocks.into_iter().flatten().collect();
        assert_eq!(flat, vec![TxIndex::from(0usize)]);
    }

    #[test]
    fn vsize_cap_respected_except_for_last_block() {
        let big = u64::from(VSize::MAX_BLOCK) - 100;
        // Three "fills the rest of a block" sized txs, one block window.
        // Final block has no cap, so all three end up in it when the
        // request is one block deep.
        let txs = vec![
            snap_tx(1, 1_000, big),
            snap_tx(2, 900, big),
            snap_tx(3, 800, big),
        ];
        let included = [0, 0, 0];
        let one_block = Partitioner::partition(&txs, &included, 1);
        assert_eq!(one_block.len(), 1);
        assert_eq!(one_block[0].len(), 3, "final block ignores vsize cap");

        // With three slots, the first two get one tx each, last block
        // soaks up the rest.
        let three_blocks = Partitioner::partition(&txs, &included, 3);
        assert_eq!(three_blocks[0].len(), 1);
        assert_eq!(three_blocks[1].len(), 1);
        assert_eq!(three_blocks[2].len(), 1);
    }

    #[test]
    fn txid_breaks_ties_within_same_rate() {
        // Identical rate, distinct txids: order must follow ascending txid.
        let txs = vec![
            snap_tx(0x20, 100, 100),
            snap_tx(0x10, 100, 100),
            snap_tx(0x30, 100, 100),
        ];
        let blocks = Partitioner::partition(&txs, &[0, 0, 0], 1);
        let txids: Vec<u8> = blocks[0]
            .iter()
            .map(|i| txs[i.as_usize()].txid[0])
            .collect();
        assert_eq!(txids, vec![0x10, 0x20, 0x30]);
    }
}
