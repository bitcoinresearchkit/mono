use crate::CheckedSub;

use super::{DeltaOp, exact_diff};

/// Rolling average from cumulative: `(cum[h] - cum[start - 1]) / (h - start + 1)`, subtracted
/// exactly before converting.
#[derive(Clone, Copy)]
pub struct DeltaAvg;

impl<S, T> DeltaOp<S, T> for DeltaAvg
where
    S: CheckedSub + PartialOrd + Clone + Into<f64> + Default,
    T: From<f64>,
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
    fn combine(current: S, ago: S, count: usize) -> T {
        if count == 0 {
            T::from(0.0)
        } else {
            T::from(exact_diff(current, ago) / count as f64)
        }
    }
}
