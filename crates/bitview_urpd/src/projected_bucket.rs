use bitview_primitives::CentsCompact;
use brk_types::Sats;

/// One price projected into the same age filters for raw and weighted supply.
pub struct ProjectedBucket<const N: usize, const C: usize> {
    pub price: CentsCompact,
    pub raw: [Sats; C],
    pub weighted: [[Sats; N]; C],
}
