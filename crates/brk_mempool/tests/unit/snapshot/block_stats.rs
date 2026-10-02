use brk_types::{Txid, get_weighted_percentile};

use super::*;
use crate::snapshot::partition::partition;

#[test]
fn block_statistics_match_sorted_weighted_ranks() {
    let txs: Vec<_> = (0..128u64)
        .map(|i| SnapTx {
            txid: Txid::COINBASE,
            fee: Sats::from(i * 100),
            vsize: VSize::new(60 + i * 1100),
            weight: Default::default(),
            size: i * 13,
            chunk_rate: FeeRate::from((i * 7 % 19) as f64),
            parents: Default::default(),
            children: Default::default(),
        })
        .collect();
    let block: Vec<_> = (0..txs.len()).rev().map(TxIndex::from).collect();
    let mut blocks = vec![block.clone(), block, vec![]];
    // Exercise the real producer's descending order, ties and overflow block,
    // alongside subsets of the same published transactions.
    blocks.extend(partition(&txs, &vec![0; txs.len()], 7));
    blocks.push((0..txs.len()).step_by(7).map(TxIndex::from).collect());
    let stats = BlockStats::for_blocks(&blocks, &txs);
    for (i, (stat, block)) in stats.iter().zip(&blocks).enumerate() {
        let entries: Vec<_> = block
            .iter()
            .filter_map(|index| txs.get(index.as_usize()))
            .collect();
        let mut rates: Vec<_> = entries.iter().map(|tx| (tx.chunk_rate, tx.vsize)).collect();
        rates.sort_unstable_by_key(|&(rate, _)| rate);
        let ranks = if i == 0 {
            CORE_PERCENTILES
        } else {
            PROJECTED_PERCENTILES
        };
        let expected = if rates.is_empty() {
            [FeeRate::default(); 7]
        } else {
            ranks.map(|rank| get_weighted_percentile(&rates, rank))
        };
        assert_eq!(stat.fee_range, expected, "block {i}");
        assert_eq!(stat.tx_count, entries.len() as u32);
        assert_eq!(stat.total_fee, entries.iter().map(|tx| tx.fee).sum());
        assert_eq!(stat.total_vsize, entries.iter().map(|tx| tx.vsize).sum());
        assert_eq!(
            stat.total_size,
            entries.iter().map(|tx| tx.size).sum::<u64>()
        );
    }
}
