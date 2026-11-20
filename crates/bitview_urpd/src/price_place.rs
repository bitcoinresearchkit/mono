use std::{array, sync::LazyLock};

use bitview_primitives::CentsCompact;
use bitview_types::UrpdAggregation;
use brk_types::Cents;

/// The union of all interval boundaries is the coarsest grid that can answer every request.
/// The ordinal of a boundary is the price part of a persisted cell key. Changing these
/// boundaries requires a new URPD history format revision.
pub(crate) static GRID: LazyLock<PriceGrid> = LazyLock::new(PriceGrid::new);

// The price field of a persisted cell key has room for the 314,727 supported places.
pub(crate) const PLACE_BITS: u32 = 19;
pub(crate) const PLACE_MASK: u32 = (1 << PLACE_BITS) - 1;

pub(crate) struct PriceGrid {
    edges: Vec<u32>,
    rows: [Vec<Row>; UrpdAggregation::ALL.len()],
}

/// A run of stored cells with the same raw valuation and requested bucket.
pub(crate) struct Row {
    pub raw: CentsCompact,
    pub bucket: CentsCompact,
    pub raw_start: u32,
    pub start: u32,
    pub end: u32,
}

impl PriceGrid {
    fn new() -> Self {
        let mut edges = Vec::new();
        for agg in UrpdAggregation::ALL {
            let mut lower = 0;
            while lower < u64::from(u32::MAX) {
                edges.push(lower as u32);
                lower = agg.interval(Cents::new(lower)).1;
            }
        }
        edges.sort_unstable();
        edges.dedup();
        assert!(edges.len() <= PLACE_MASK as usize + 1, "URPD grid exceeds its key field");
        let rows = array::from_fn(|i| Self::bucket_rows(&edges, UrpdAggregation::ALL[i]));
        Self { edges, rows }
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn place(&self, cents: u32) -> u32 {
        (self.edges.partition_point(|&edge| edge <= cents) - 1) as u32
    }

    pub fn rows(&self, agg: UrpdAggregation) -> &[Row] {
        &self.rows[agg as usize]
    }

    fn bucket_rows(edges: &[u32], agg: UrpdAggregation) -> Vec<Row> {
        let mut rows = Vec::new();
        let mut raw_start = 0;
        let (mut raw, mut raw_end) = (0, 0);
        let (mut label, mut bucket_end) = (0, 0);
        let mut end = 0;
        while end < edges.len() {
            let start = end;
            let lower = u64::from(edges[start]);
            if lower == raw_end {
                (raw, raw_end) = UrpdAggregation::Raw.interval(Cents::new(lower));
                raw_start = start as u32;
            }
            if lower == bucket_end {
                (label, bucket_end) = agg.interval(Cents::new(lower));
            }
            let upper = raw_end.min(bucket_end);
            end += edges[start..].partition_point(|&p| u64::from(p) < upper);
            rows.push(Row {
                raw: CentsCompact::new(raw as u32),
                bucket: CentsCompact::new(label as u32),
                raw_start,
                start: start as u32,
                end: end as u32,
            });
        }
        rows
    }
}
