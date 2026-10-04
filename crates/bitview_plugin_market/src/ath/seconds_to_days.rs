use bitview_primitives::{Days, Seconds};
use brk_types::ONE_DAY_IN_SEC_F64;
use vecdb::UnaryTransform;

pub(super) struct SecondsToDays;

impl UnaryTransform<Seconds, Days> for SecondsToDays {
    fn apply(seconds: Seconds) -> Days {
        Days::from(*seconds as f64 / ONE_DAY_IN_SEC_F64)
    }
}
