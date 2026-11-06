use brk_types::{FeeRate, MempoolBlock, Sats, VSize, Weight, weighted_percentiles};

use super::{SnapTx, TxIndex};

/// Block 0 mirrors Core's `getblocktemplate`, so the full 0..100 range
/// is exact and worth surfacing.
const CORE_PERCENTILES: [u64; 7] = [0, 10, 25, 50, 75, 90, 100];

/// Blocks 1..N are a projection. Tighten to 5..95 so a single
/// stale-GBT leftover or CPFP orphan doesn't blow out the min/max
/// columns of an otherwise tightly clustered fee tier.
const PROJECTED_PERCENTILES: [u64; 7] = [5, 10, 25, 50, 75, 90, 95];

/// Per-block aggregate stats for a projected block.
///
/// `block_stats[0]` mirrors Bitcoin Core's `getblocktemplate` - the
/// node's actual next-block selection. `fee_range` spans the full
/// 0..100 percentiles.
///
/// `block_stats[1..]` are the package-aware projection (whole chunks by
/// descending feerate within Core's budgets), a client-facing fee-tier
/// gradient: later blocks depend on transactions not yet broadcast. Their
/// `fee_range` is clipped to 5..95 percentiles so a single stale-GBT
/// leftover or CPFP orphan doesn't dominate the min/max columns.
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
    /// Stats for every projected block, in order: block 0 (Core's template, each member at its
    /// chunk's feerate) over the exact 0..100 percentiles, the rest (each member at the feerate
    /// it was packed at) over the clipped 5..95 range that hides CPFP / stale-GBT outliers.
    pub fn for_blocks(
        block0: &[TxIndex],
        rest: &[Vec<(TxIndex, FeeRate)>],
        txs: &[SnapTx],
    ) -> Vec<Self> {
        let block0 = block0.iter().map(|&i| (i, txs[i.as_usize()].chunk_rate));
        let mut stats = vec![Self::compute(block0, txs, CORE_PERCENTILES)];
        // Projected blocks descend by rate. Read backwards so sort's ascending-run check
        // avoids sorting them again.
        stats.extend(rest.iter().map(|block| {
            Self::compute(block.iter().rev().copied(), txs, PROJECTED_PERCENTILES)
        }));
        stats
    }

    /// Vsize-weighted percentile distribution over the members' feerates -
    /// matches mempool.space's `feeRange` semantics where each tx's
    /// contribution scales with its vsize, so a tiny outlier rate
    /// only counts for its own vsize fraction.
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
