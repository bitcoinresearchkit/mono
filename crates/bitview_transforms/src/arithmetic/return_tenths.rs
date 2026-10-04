use bitview_primitives::Float32;
use vecdb::UnaryTransform;

/// A constant: `V` tenths.
pub struct ReturnTenths<const V: i16>;

impl<S, const V: i16> UnaryTransform<S, Float32> for ReturnTenths<V> {
    #[inline(always)]
    fn apply(_: S) -> Float32 {
        Float32::new(f32::from(V) / 10.0)
    }
}
