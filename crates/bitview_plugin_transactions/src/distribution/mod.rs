mod per_tx;

pub use per_tx::PerTxDistribution;

use bitview_compute::{ComputedVecValue, DistributionStats, NumericValue};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, PerBlock};
use brk_error::Result;
use brk_types::{VSize, weighted_percentiles};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode, Version, WritableVec};

/// Share of a block's vsize at each end whose extreme rates bound the effective fee rate range.
const EDGE_SHARE: f64 = 0.02;

/// One block's statistics in [`DistributionStats::as_mut_array`] order: min, max, p10,
/// p25, median, p75, p90.
pub type Stats<T> = [T; 7];

/// Per-block distributions of a per-transaction value, statistic first, ids `{name}_{stat}`.
#[derive(Deref, DerefMut, Traversable)]
pub struct TxDistribution<T, M: StorageMode = Rw>(pub DistributionStats<PerBlock<T, M>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> TxDistribution<T>
where
    T: NumericValue + JsonSchema + Ord,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        Ok(Self(DistributionStats::try_from_fn(|stat| {
            PerBlock::import(db, &format!("{name}_{stat}"), version, indexes)
        })?))
    }

    pub fn push(&mut self, stats: Stats<T>) {
        for (stat, value) in self.0.as_mut_array().into_iter().zip(stats) {
            stat.height.push(value);
        }
    }
}

/// Values counted equally, as mempool.space's fee percentiles: sorted, min the first, max the
/// last and percentile `p` at `sorted[max(0, floor(n * p) - 1)]`.
pub fn counted<T: NumericValue + Ord>(values: &mut [T]) -> Stats<T> {
    values.sort_unstable();
    let (Some(&min), Some(&max)) = (values.first(), values.last()) else {
        return undefined();
    };
    let at = |p: f64| values[((values.len() as f64 * p) as usize).saturating_sub(1)];
    [min, max, at(0.10), at(0.25), at(0.50), at(0.75), at(0.90)]
}

/// Rates weighted by vsize, with Bitcoin Core's `getblockstats` rank rule: the first rate whose
/// cumulative vsize reaches `p` of the total. Min and max follow mempool.space's effective fee
/// heuristic: the lower of the 1st percentile and the lowest rate among the transactions
/// overlapping the block's last 2% of vsize (block order), the higher of the 99th percentile
/// and the highest among those overlapping its first 2%.
pub fn weighted<T: NumericValue + Ord>(block: &mut [(T, VSize)]) -> Stats<T> {
    if block.is_empty() {
        return undefined();
    }
    let total: u64 = block.iter().map(|&(_, vsize)| u64::from(vsize)).sum();
    let edge = total as f64 * EDGE_SHARE;
    let head_max = edge_rate(block.iter().copied(), edge, Ord::max);
    let tail_min = edge_rate(block.iter().rev().copied(), edge, Ord::min);
    block.sort_unstable();
    let [p1, p10, p25, median, p75, p90, p99] =
        weighted_percentiles(block, [1, 10, 25, 50, 75, 90, 99]);
    [
        p1.min(tail_min),
        p99.max(head_max),
        p10,
        p25,
        median,
        p75,
        p90,
    ]
}
/// The extreme rate, by `pick`, of the transactions overlapping the first `edge` vbytes of `txs`
/// (at least the first one).
fn edge_rate<T: Copy>(
    mut txs: impl Iterator<Item = (T, VSize)>,
    edge: f64,
    pick: impl Fn(T, T) -> T,
) -> T {
    let (mut extreme, vsize) = txs.next().unwrap();
    let mut before = u64::from(vsize);
    for (rate, vsize) in txs {
        if before as f64 >= edge {
            break;
        }
        extreme = pick(extreme, rate);
        before += u64::from(vsize);
    }
    extreme
}

/// No values: the type's undefined value, null for rates and zero for amounts.
fn undefined<T: NumericValue>() -> Stats<T> {
    [T::from(f64::NAN); 7]
}
