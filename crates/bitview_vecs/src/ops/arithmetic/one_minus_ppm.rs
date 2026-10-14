use bitview_primitives::PartsPerMillion32;
use vecdb::UnaryTransform;

pub struct OneMinusPpm;

impl UnaryTransform<PartsPerMillion32, PartsPerMillion32> for OneMinusPpm {
    #[inline(always)]
    fn apply(value: PartsPerMillion32) -> PartsPerMillion32 {
        PartsPerMillion32::ONE - value
    }
}
