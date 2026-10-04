use bitview_primitives::{CoinDays, CoinYears, Days, Years};
use vecdb::UnaryTransform;

pub struct DaysToYears;

impl UnaryTransform<Days, Years> for DaysToYears {
    #[inline(always)]
    fn apply(value: Days) -> Years {
        Years::new(*value / 365.0)
    }
}

impl UnaryTransform<CoinDays, CoinYears> for DaysToYears {
    #[inline(always)]
    fn apply(value: CoinDays) -> CoinYears {
        CoinYears::new(*value / 365.0)
    }
}
