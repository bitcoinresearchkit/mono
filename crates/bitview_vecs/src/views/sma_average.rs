use bitview_primitives::StoredU64;
use brk_types::Cents;
use vecdb::DeltaOp;

/// Exact integer-cent average over a cumulative price source.
pub struct SmaAverage;

impl DeltaOp<StoredU64, Cents> for SmaAverage {
    fn ago_index(start: usize) -> Option<usize> {
        start.checked_sub(1)
    }
    fn ago_default() -> StoredU64 {
        StoredU64::from(0_u64)
    }
    fn count(index: usize, start: usize) -> usize {
        index - start + 1
    }
    fn combine(current: StoredU64, previous: StoredU64, count: usize) -> Cents {
        Cents::new((u64::from(current) - u64::from(previous)) / count as u64)
    }
}
