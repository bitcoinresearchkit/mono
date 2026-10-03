use bitview_primitives::OHLCCents;
use brk_types::Cents;
use vecdb::UnaryTransform;

pub struct OhlcCentsToHighCents;

impl UnaryTransform<OHLCCents, Cents> for OhlcCentsToHighCents {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> Cents {
        *cents.high
    }
}
