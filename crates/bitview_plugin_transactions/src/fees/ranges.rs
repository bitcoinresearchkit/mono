//! Transactions by fee-rate range, as counts at their own rate and as block space at the
//! effective rate that accounts for the related transactions they confirm with.

use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_types::{FeeRate, Sats, VSize, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

macro_rules! define_ranges {
    ($($field:ident,)+) => {
        /// One value per fee-rate range, in sat/vB: each range holds its lower edge.
        #[derive(Traversable)]
        pub struct ByRange<T> {
            $(pub $field: T,)+
        }

        impl<T> ByRange<T> {
            fn try_from_fn(mut create: impl FnMut(&str) -> Result<T>) -> Result<Self> {
                Ok(Self {
                    $($field: create(stringify!($field).trim_start_matches('_'))?,)+
                })
            }

            fn all_mut(&mut self) -> [&mut T; RANGES] {
                [$(&mut self.$field,)+]
            }
        }
    };
}

define_ranges! {
    under_0p1_sat_vb,
    _0p1_to_0p25_sat_vb,
    _0p25_to_0p5_sat_vb,
    _0p5_to_1_sat_vb,
    _1_to_2_sat_vb,
    _2_to_5_sat_vb,
    _5_to_10_sat_vb,
    _10_to_25_sat_vb,
    _25_to_50_sat_vb,
    _50_to_100_sat_vb,
    _100_to_250_sat_vb,
    _250_to_500_sat_vb,
    _500_to_1k_sat_vb,
    over_1k_sat_vb,
}

pub(crate) const RANGES: usize = 14;

/// The ranges' upper edges, in twentieths of a sat/vB: a fee and a size compare against them
/// exactly.
const EDGES_TWENTIETHS: [u64; RANGES - 1] = [
    2, 5, 10, 20, 40, 100, 200, 500, 1_000, 2_000, 5_000, 10_000, 20_000,
];
/// The same edges in sat/vB, for effective rates.
const EDGES: [f64; RANGES - 1] = {
    let mut edges = [0.0; RANGES - 1];
    let mut i = 0;
    while i < edges.len() {
        edges[i] = EDGES_TWENTIETHS[i] as f64 / 20.0;
        i += 1;
    }
    edges
};

#[inline]
fn raw_range(fee: u64, vsize: u64) -> usize {
    EDGES_TWENTIETHS.partition_point(|&edge| fee * 20 >= vsize * edge)
}

#[inline]
fn effective_range(rate: f64) -> usize {
    EDGES.partition_point(|&edge| rate >= edge)
}

/// One block's non-coinbase transactions by range.
#[derive(Default)]
pub(crate) struct BlockRanges {
    zero_fee: u64,
    counts: [u64; RANGES],
    sizes: [u64; RANGES],
}

impl BlockRanges {
    /// Adds a transaction by its fee, virtual size and effective rate.
    #[inline]
    pub(crate) fn add(&mut self, fee: Sats, vsize: VSize, effective: FeeRate) {
        let (fee, vsize) = (u64::from(fee), u64::from(vsize));
        if fee == 0 {
            self.zero_fee += 1;
        } else {
            self.counts[raw_range(fee, vsize)] += 1;
        }
        self.sizes[effective_range(f64::from(effective))] += vsize;
    }
}

#[derive(Traversable)]
pub struct Ranges<M: StorageMode = Rw> {
    /// Non-coinbase transactions paying no fee at all.
    #[traversable(
        wrap = "zero_fee",
        rename = "tx_count",
        lead = "Zero-fee transactions."
    )]
    pub zero_fee: PerBlockCumulativeRolling<Count, M>,
    /// Non-coinbase transactions whose own fee divided by their virtual size falls in each range.
    #[traversable(lead = "Transactions by the fee rate they paid themselves.")]
    pub tx_count_by_fee_rate: ByRange<PerBlockCumulativeRolling<Count, M>>,
    /// The virtual size of non-coinbase transactions whose effective fee rate falls in each range,
    /// the lowest range holding those at zero.
    #[traversable(lead = "Block space by effective fee rate.")]
    pub vsize_by_effective_fee_rate: ByRange<PerBlockCumulativeRolling<VSize, M>>,
}

impl Ranges {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        indexes: &IndexSources,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            zero_fee: PerBlockCumulativeRolling::import(
                db,
                "zero_fee_tx_count",
                version,
                indexes,
                window_starts,
            )?,
            tx_count_by_fee_rate: ByRange::try_from_fn(|range| {
                PerBlockCumulativeRolling::import(
                    db,
                    &format!("fee_rate_{range}_tx_count"),
                    version,
                    indexes,
                    window_starts,
                )
            })?,
            vsize_by_effective_fee_rate: ByRange::try_from_fn(|range| {
                let name = format!("effective_fee_rate_{range}_tx_vsize");
                PerBlockCumulativeRolling::import(db, &name, version, indexes, window_starts)
            })?,
        })
    }

    pub(crate) fn stored_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self {
            zero_fee,
            tx_count_by_fee_rate,
            vsize_by_effective_fee_rate,
        } = self;
        [zero_fee.stored_mut()]
            .into_iter()
            .chain(
                tx_count_by_fee_rate
                    .all_mut()
                    .map(|range| range.stored_mut()),
            )
            .chain(
                vsize_by_effective_fee_rate
                    .all_mut()
                    .map(|range| range.stored_mut()),
            )
            .collect()
    }

    pub(crate) fn push_block(&mut self, block: &BlockRanges) {
        self.zero_fee.push_block(Count::from(block.zero_fee));
        for (range, &count) in self
            .tx_count_by_fee_rate
            .all_mut()
            .into_iter()
            .zip(&block.counts)
        {
            range.push_block(Count::from(count));
        }
        for (range, &size) in self
            .vsize_by_effective_fee_rate
            .all_mut()
            .into_iter()
            .zip(&block.sizes)
        {
            range.push_block(VSize::from(size));
        }
    }
}
