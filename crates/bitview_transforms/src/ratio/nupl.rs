use brk_types::{PartsPerMillionSigned32, PriceRatio};
use vecdb::UnaryTransform;

pub struct MvrvToNupl;

impl UnaryTransform<PriceRatio, PartsPerMillionSigned32> for MvrvToNupl {
    #[inline(always)]
    fn apply(mvrv: PriceRatio) -> PartsPerMillionSigned32 {
        PartsPerMillionSigned32::from(1.0 - 1.0 / f64::from(mvrv))
    }
}
