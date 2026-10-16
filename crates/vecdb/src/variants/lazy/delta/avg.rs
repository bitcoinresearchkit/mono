use crate::CheckedSub;

use super::{DeltaOp, exact_diff};

/// Rolling average from cumulative: `(cum[h] - cum[start - 1]) / (h - start + 1)`, subtracted
/// exactly before converting.
#[derive(Clone, Copy)]
pub struct DeltaAvg;

impl<S, T> DeltaOp<S, T> for DeltaAvg
where
    S: CheckedSub + PartialOrd + Clone + Into<f64>,
    T: From<f64>,
{
    #[inline]
    fn combine(current: S, before: S, count: usize) -> T {
        if count == 0 {
            T::from(0.0)
        } else {
            T::from(exact_diff(current, before) / count as f64)
        }
    }
}
