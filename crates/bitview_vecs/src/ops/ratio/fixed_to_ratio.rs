use vecdb::UnaryTransform;

/// A fixed-point ratio as its public value: a plain ratio or a multiple.
pub struct FixedToRatio;

impl<T, O> UnaryTransform<T, O> for FixedToRatio
where
    T: Into<f32>,
    O: From<f32>,
{
    #[inline(always)]
    fn apply(value: T) -> O {
        O::from(value.into())
    }
}
