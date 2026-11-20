use bitview_primitives::CentsCompact;
use brk_types::Sats;

/// One price projected into the same age filters for raw and weighted supply.
#[derive(Clone, Copy)]
pub struct ProjectedBucket<const N: usize, const C: usize> {
    /// Four-significant-digit price used for valuation.
    pub price: CentsCompact,
    /// Price label of the requested interval.
    pub label: CentsCompact,
    pub raw: [Sats; C],
    pub weighted: [[Sats; N]; C],
}
