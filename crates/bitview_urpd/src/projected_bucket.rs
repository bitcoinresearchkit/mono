use brk_types::{CentsCompact, Sats};

/// One occupied price bucket projected into raw supply and caller-selected weights.
pub struct ProjectedBucket<const N: usize> {
    pub price: CentsCompact,
    pub raw: Sats,
    pub weighted: [Sats; N],
}
