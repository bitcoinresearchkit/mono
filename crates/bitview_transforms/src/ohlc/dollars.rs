use bitview_primitives::{OHLCCents, OHLCDollars};
use vecdb::UnaryTransform;

pub struct OhlcCentsToDollars;

impl UnaryTransform<OHLCCents, OHLCDollars> for OhlcCentsToDollars {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> OHLCDollars {
        OHLCDollars::from(cents)
    }
}
