use bitview_primitives::Percent;
use vecdb::UnaryTransform;

pub struct FixedToPercent;

impl<T> UnaryTransform<T, Percent> for FixedToPercent
where
    T: Into<f32>,
{
    #[inline(always)]
    fn apply(value: T) -> Percent {
        Percent::new(value.into() * 100.0)
    }
}
