use crate::CheckedSub;

use super::{DeltaOp, exact_diff};

/// Growth over a window: `(source[h] - source[start - 1]) / source[start - 1]`, from the value
/// before the window (zero for a window from 0); undefined (NaN, or infinite) when that value is
/// zero.
#[derive(Clone, Copy)]
pub struct DeltaRate;

impl<S, B> DeltaOp<S, B> for DeltaRate
where
    S: CheckedSub + PartialOrd + Into<f64> + Default + Clone,
    B: From<f64>,
{
    #[inline]
    fn ago_index(start: usize) -> Option<usize> {
        start.checked_sub(1)
    }

    #[inline]
    fn ago_default() -> S {
        S::default()
    }

    #[inline]
    fn count(h: usize, start: usize) -> usize {
        h - start + 1
    }

    #[inline]
    fn combine(current: S, ago: S, _count: usize) -> B {
        let base: f64 = ago.clone().into();
        B::from(exact_diff(current, ago) / base)
    }
}
