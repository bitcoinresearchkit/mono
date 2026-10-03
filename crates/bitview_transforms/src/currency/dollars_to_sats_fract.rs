use bitview_primitives::SatsFract;
use brk_types::Dollars;
use vecdb::UnaryTransform;

pub struct DollarsToSatsFract;

impl UnaryTransform<Dollars, SatsFract> for DollarsToSatsFract {
    #[inline(always)]
    fn apply(dollars: Dollars) -> SatsFract {
        SatsFract::ONE_BTC / dollars
    }
}
