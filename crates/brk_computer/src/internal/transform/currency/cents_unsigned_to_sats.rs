use brk_types::{Cents, Dollars, Sats};
use vecdb::{UnaryTransform, unlikely};

pub struct CentsUnsignedToSats;

impl UnaryTransform<Cents, Sats> for CentsUnsignedToSats {
    #[inline(always)]
    fn apply(cents: Cents) -> Sats {
        if unlikely(cents.is_nan()) {
            panic!("Cents::NAN cannot be converted to whole Sats");
        }
        let dollars = Dollars::from(cents);
        if dollars == Dollars::ZERO {
            Sats::ZERO
        } else {
            Sats::ONE_BTC / dollars
        }
    }
}
