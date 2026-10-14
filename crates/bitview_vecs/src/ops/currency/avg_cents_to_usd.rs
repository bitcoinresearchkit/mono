use bitview_primitives::CentsFract;
use brk_types::Dollars;
use vecdb::UnaryTransform;

pub struct AvgCentsToUsd;

impl UnaryTransform<CentsFract, Dollars> for AvgCentsToUsd {
    #[inline(always)]
    fn apply(cents: CentsFract) -> Dollars {
        Dollars::from(f64::from(cents) / 100.0)
    }
}
