use bitview_primitives::Float32;
use vecdb::UnaryTransform;

pub struct ThsToPhs;

impl UnaryTransform<Float32, Float32> for ThsToPhs {
    #[inline(always)]
    fn apply(ths: Float32) -> Float32 {
        (*ths * 1000.0).into()
    }
}
