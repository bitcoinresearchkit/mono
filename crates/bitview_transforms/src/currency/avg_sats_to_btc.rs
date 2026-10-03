use bitview_primitives::StoredF32;
use brk_types::{Bitcoin, Sats};
use vecdb::UnaryTransform;

pub struct AvgSatsToBtc;

impl UnaryTransform<StoredF32, Bitcoin> for AvgSatsToBtc {
    #[inline(always)]
    fn apply(sats: StoredF32) -> Bitcoin {
        Bitcoin::from(f64::from(sats) / Sats::ONE_BTC_U128 as f64)
    }
}
