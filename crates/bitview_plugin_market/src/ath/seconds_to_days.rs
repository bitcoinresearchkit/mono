use bitview_primitives::{Days, Seconds};
use brk_types::ONE_DAY_IN_SEC_F64;
use vecdb::UnaryTransform;

/// Seconds since the ATH before the first price, which has none: null days.
pub(super) const UNPRICED: Seconds = Seconds::MAX;

pub(super) struct SecondsToDays;

impl UnaryTransform<Seconds, Days> for SecondsToDays {
    fn apply(seconds: Seconds) -> Days {
        if seconds == UNPRICED {
            Days::NAN
        } else {
            Days::from(*seconds as f64 / ONE_DAY_IN_SEC_F64)
        }
    }
}
