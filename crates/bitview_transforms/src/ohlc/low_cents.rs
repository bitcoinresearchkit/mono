use bitview_primitives::OHLCCents;
use brk_types::Cents;
use vecdb::UnaryTransform;

pub struct OhlcCentsToLowCents;

impl UnaryTransform<OHLCCents, Cents> for OhlcCentsToLowCents {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> Cents {
        *cents.low
    }
}
