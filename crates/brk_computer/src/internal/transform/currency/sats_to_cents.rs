use brk_types::{Cents, Sats};
use vecdb::{BinaryTransform, unlikely};

pub struct SatsToCents;

impl BinaryTransform<Sats, Cents, Cents> for SatsToCents {
    #[inline(always)]
    fn apply(sats: Sats, price_cents: Cents) -> Cents {
        if unlikely(price_cents.is_nan()) {
            Cents::NAN
        } else {
            Cents::from(sats.as_u128() * price_cents.as_u128() / Sats::ONE_BTC_U128)
        }
    }
}
