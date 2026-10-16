use crate::CheckedSub;

use super::DeltaOp;

/// Rolling sum from cumulative: `cum[h] - cum[start - 1]`
#[derive(Clone, Copy)]
pub struct DeltaSub;

impl<T> DeltaOp<T, T> for DeltaSub
where
    T: CheckedSub + Default,
{
    #[inline]
    fn combine(current: T, before: T, _count: usize) -> T {
        current.checked_sub(before).unwrap_or_default()
    }
}
