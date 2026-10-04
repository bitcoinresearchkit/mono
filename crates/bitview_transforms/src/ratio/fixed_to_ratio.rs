use bitview_primitives::Ratio;
use vecdb::UnaryTransform;

pub struct FixedToRatio;

impl<T> UnaryTransform<T, Ratio> for FixedToRatio
where
    f32: From<T>,
{
    #[inline(always)]
    fn apply(value: T) -> Ratio {
        Ratio::new(f32::from(value))
    }
}
