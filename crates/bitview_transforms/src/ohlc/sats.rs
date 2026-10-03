use bitview_primitives::{Close, High, Low, OHLCCents, OHLCSats, Open};
use vecdb::UnaryTransform;

use crate::CentsUnsignedToSats;

pub struct OhlcCentsToSats;

impl UnaryTransform<OHLCCents, OHLCSats> for OhlcCentsToSats {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> OHLCSats {
        OHLCSats {
            open: Open::new(CentsUnsignedToSats::apply(*cents.open)),
            high: High::new(CentsUnsignedToSats::apply(*cents.low)),
            low: Low::new(CentsUnsignedToSats::apply(*cents.high)),
            close: Close::new(CentsUnsignedToSats::apply(*cents.close)),
        }
    }
}
