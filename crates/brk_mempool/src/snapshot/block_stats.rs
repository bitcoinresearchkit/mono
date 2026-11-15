use brk_types::{FeeRate, MempoolBlock, Sats, VSize, Weight, weighted_percentiles};

use super::{SnapTx, TxIndex};

/// Block 0 is the next block, packed from transactions all already broadcast, so the full 0..100
/// range is worth surfacing.
const NEXT_BLOCK_PERCENTILES: [u64; 7] = [0, 10, 25, 50, 75, 90, 100];

/// Blocks 1..N are a projection: later blocks depend on transactions not yet broadcast. Tightened
/// to 5..95 so a single CPFP orphan doesn't blow out the min/max columns of an otherwise tightly
/// clustered fee tier.
const PROJECTED_PERCENTILES: [u64; 7] = [5, 10, 25, 50, 75, 90, 95];

/// Per-block aggregate stats for a projected block.
#[derive(Debug, Clone, Default)]
pub struct BlockStats {
    pub tx_count: u32,
    pub total_size: u64,
    /// The transactions' actual weight (what Core's budget counts).
    pub total_weight: Weight,
    pub total_fee: Sats,
    pub fee_range: [FeeRate; 7],
}

impl BlockStats {
    /// Stats for every projected block, in order, each member at the feerate it was packed at:
    /// block 0 over the exact 0..100 percentiles, the rest over the clipped 5..95 range.
    pub fn for_blocks(blocks: &[Vec<(TxIndex, FeeRate)>], txs: &[SnapTx]) -> Vec<Self> {
        // Packed blocks descend by rate. Read backwards so sort's ascending-run check avoids
        // sorting them again.
        blocks
            .iter()
            .enumerate()
            .map(|(i, block)| {
                let percentiles = if i == 0 { NEXT_BLOCK_PERCENTILES } else { PROJECTED_PERCENTILES };
                Self::compute(block.iter().rev().copied(), txs, percentiles)
            })
            .collect()
    }

    /// Vsize-weighted percentile distribution over the members' feerates, as Core's
    /// `getblockstats` weighs them (mempool.space's `feeRange` counts transactions instead): a
    /// tiny outlier rate only counts for its own vsize fraction.
    fn compute(
        block: impl ExactSizeIterator<Item = (TxIndex, FeeRate)>,
        txs: &[SnapTx],
        percentiles: [u64; 7],
    ) -> Self {
        let mut total_fee = Sats::default();
        let mut total_weight = Weight::default();
        let mut total_size: u64 = 0;
        let mut rates: Vec<(FeeRate, VSize)> = Vec::with_capacity(block.len());

        for (tx_index, rate) in block {
            let t = &txs[tx_index.as_usize()];
            total_fee += t.fee;
            total_weight += t.weight;
            total_size += t.size;
            rates.push((rate, t.vsize));
        }

        rates.sort_unstable_by_key(|(r, _)| *r);

        let fee_range: [FeeRate; 7] = if rates.is_empty() {
            [FeeRate::default(); 7]
        } else {
            weighted_percentiles(&rates, percentiles)
        };

        Self {
            tx_count: rates.len() as u32,
            total_size,
            total_weight,
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
            s.total_weight,
            s.total_fee,
            s.fee_range,
        )
    }
}
