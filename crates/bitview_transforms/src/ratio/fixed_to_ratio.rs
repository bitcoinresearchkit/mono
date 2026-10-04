use bitview_primitives::Ratio;
use vecdb::UnaryTransform;

pub struct FixedToRatio;

impl<T> UnaryTransform<T, Ratio> for FixedToRatio
where
    T: Into<f32>,
{
    #[inline(always)]
    fn apply(value: T) -> Ratio {
        Ratio::new(value.into())
    }
}
