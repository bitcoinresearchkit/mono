use bitview_primitives::SatsFract;
use brk_types::{Bitcoin, Sats};
use vecdb::UnaryTransform;

pub struct AvgSatsToBtc;

impl UnaryTransform<SatsFract, Bitcoin> for AvgSatsToBtc {
    #[inline(always)]
    fn apply(sats: SatsFract) -> Bitcoin {
        Bitcoin::from(f64::from(sats) / Sats::ONE_BTC_U128 as f64)
    }
}
