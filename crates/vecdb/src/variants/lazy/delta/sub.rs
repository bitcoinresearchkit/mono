use crate::CheckedSub;

use super::DeltaOp;

/// Rolling sum from cumulative: `cum[h] - cum[start - 1]`. A cumulative never decreases, so an
/// underflow means wrong data: debug builds panic, release builds give `T::default()`.
#[derive(Clone, Copy)]
pub struct DeltaSub;

impl<T> DeltaOp<T, T> for DeltaSub
where
    T: CheckedSub + Default,
{
    #[inline]
    fn combine(current: T, before: T, _count: usize) -> T {
        let sum = current.checked_sub(before);
        debug_assert!(sum.is_some(), "rolling sum underflow: its cumulative decreased");
        sum.unwrap_or_default()
    }
}
