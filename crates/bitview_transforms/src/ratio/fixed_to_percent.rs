use bitview_primitives::StoredF32;
use vecdb::UnaryTransform;

pub struct FixedToPercent;

impl<T> UnaryTransform<T, StoredF32> for FixedToPercent
where
    f32: From<T>,
{
    #[inline(always)]
    fn apply(value: T) -> StoredF32 {
        StoredF32::from(f32::from(value) * 100.0)
    }
}
