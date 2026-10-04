use bitview_primitives::Percent;
use vecdb::UnaryTransform;

pub struct FixedToPercent;

impl<T> UnaryTransform<T, Percent> for FixedToPercent
where
    f32: From<T>,
{
    #[inline(always)]
    fn apply(value: T) -> Percent {
        Percent::new(f32::from(value) * 100.0)
    }
}
