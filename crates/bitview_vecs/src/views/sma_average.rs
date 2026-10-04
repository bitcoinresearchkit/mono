use brk_types::Cents;
use vecdb::DeltaOp;

/// Exact integer-cent average over a cumulative price source.
pub struct SmaAverage;

impl DeltaOp<Cents, Cents> for SmaAverage {
    fn ago_index(start: usize) -> Option<usize> {
        start.checked_sub(1)
    }
    fn ago_default() -> Cents {
        Cents::ZERO
    }
    fn count(index: usize, start: usize) -> usize {
        index - start + 1
    }
    fn combine(current: Cents, previous: Cents, count: usize) -> Cents {
        Cents::new((*current - *previous) / count as u64)
    }
}
