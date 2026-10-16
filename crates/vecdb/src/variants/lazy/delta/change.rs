use crate::CheckedSub;

use super::{DeltaOp, exact_diff};

/// Change over a window: `source[h] - source[start - 1]` (the value before the window, zero for
/// a window from 0), subtracted exactly, then converted (unsigned → signed). Over a cumulative
/// level this equals the sum of its flows in `[start, h]`.
#[derive(Clone, Copy)]
pub struct DeltaChange;

impl<S, C> DeltaOp<S, C> for DeltaChange
where
    S: CheckedSub + PartialOrd + Clone + Into<f64>,
    C: From<f64>,
{
    #[inline]
    fn combine(current: S, before: S, _count: usize) -> C {
        C::from(exact_diff(current, before))
    }
}
