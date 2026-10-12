use super::DeltaOp;

/// Delta rate (growth): `(source[h] - source[start]) / source[start]` via f64; undefined (NaN, or
/// infinite) when the starting value is zero.
#[derive(Clone, Copy)]
pub struct DeltaRate;

impl<S, B> DeltaOp<S, B> for DeltaRate
where
    S: Into<f64>,
    B: From<f64>,
{
    #[inline]
    fn combine(current: S, ago: S, _count: usize) -> B {
        let ago: f64 = ago.into();
        B::from((current.into() - ago) / ago)
    }
}
