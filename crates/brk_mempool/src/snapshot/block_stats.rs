use brk_types::{FeeRate, MempoolBlock, Sats, VSize, get_weighted_percentiles};

use super::{SnapTx, TxIndex};

/// Block 0 mirrors Core's `getblocktemplate`, so the full 0..100 range
/// is exact and worth surfacing.
const CORE_PERCENTILES: [f64; 7] = [0.0, 0.10, 0.25, 0.50, 0.75, 0.90, 1.00];

/// Blocks 1..N are a coarse projection. Tighten to 5..95 so a single
/// stale-GBT leftover or CPFP orphan doesn't blow out the min/max
/// columns of an otherwise tightly clustered fee tier.
const PROJECTED_PERCENTILES: [f64; 7] = [0.05, 0.10, 0.25, 0.50, 0.75, 0.90, 0.95];

/// Per-block aggregate stats for a projected block.
///
/// `block_stats[0]` mirrors Bitcoin Core's `getblocktemplate` - the
/// node's actual next-block selection. `fee_range` spans the full
/// 0..100 percentiles.
///
/// `block_stats[1..]` are a coarse greedy-packed projection by
/// descending chunk rate, useful as a client-facing fee-tier gradient
/// but not a prediction of what miners will include. Their `fee_range`
/// is clipped to 5..95 percentiles so a single stale-GBT leftover or
/// CPFP orphan doesn't dominate the min/max columns.
#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    pub tx_count: u32,
    pub total_size: u64,
    pub total_vsize: VSize,
    pub total_fee: Sats,
    pub fee_range: [FeeRate; 7],
}

impl BlockStats {
    /// Stats for every projected block in `blocks`, in order. `blocks[0]`
    /// uses Core's exact 0..100 percentiles. The rest use the clipped
    /// 5..95 range to hide CPFP / stale-GBT outliers.
    pub fn for_blocks(blocks: &[Vec<TxIndex>], txs: &[SnapTx]) -> Vec<Self> {
        blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                if i == 0 {
                    Self::compute(block.iter(), txs, CORE_PERCENTILES)
                } else {
                    // Projected blocks descend by rate. Read backwards so sort's
                    // ascending-run check avoids sorting them again.
                    Self::compute(block.iter().rev(), txs, PROJECTED_PERCENTILES)
                }
            })
            .collect()
    }

    /// Vsize-weighted percentile distribution over `chunk_rate` -
    /// matches mempool.space's `feeRange` semantics where each tx's
    /// contribution scales with its vsize, so a tiny outlier rate
    /// only counts for its own vsize fraction.
    fn compute<'a>(
        block: impl ExactSizeIterator<Item = &'a TxIndex>,
        txs: &[SnapTx],
        percentiles: [f64; 7],
    ) -> Self {
        let mut total_fee = Sats::default();
        let mut total_vsize = VSize::default();
        let mut total_size: u64 = 0;
        let mut rates: Vec<(FeeRate, VSize)> = Vec::with_capacity(block.len());

        for &tx_index in block {
            let Some(t) = txs.get(tx_index.as_usize()) else {
                continue;
            };
            total_fee += t.fee;
            total_vsize += t.vsize;
            total_size += t.size;
            rates.push((t.chunk_rate, t.vsize));
        }

        rates.sort_unstable_by_key(|(r, _)| *r);

        let fee_range: [FeeRate; 7] = if rates.is_empty() {
            [FeeRate::default(); 7]
        } else {
            get_weighted_percentiles(&rates, percentiles)
        };

        Self {
            tx_count: rates.len() as u32,
            total_size,
            total_vsize,
            total_fee,
            fee_range,
        }
    }
}

impl From<&BlockStats> for MempoolBlock {
    fn from(s: &BlockStats) -> Self {
        Self::new(
            s.tx_count,
            s.total_size,
            s.total_vsize,
            s.total_fee,
            s.fee_range,
        )
    }
}
