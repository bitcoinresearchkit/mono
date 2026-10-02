use brk_types::{Cents, Sats};
use vecdb::{BinaryTransform, unlikely};

pub struct SatsToCents;

impl BinaryTransform<Sats, Cents, Cents> for SatsToCents {
    #[inline(always)]
    fn apply(sats: Sats, price_cents: Cents) -> Cents {
        if unlikely(price_cents.is_nan()) {
            Cents::NAN
        } else if let Some(value) = u64::from(sats).checked_mul(u64::from(price_cents)) {
            Cents::from(value / Sats::ONE_BTC_U64)
        } else {
            Cents::from(sats.as_u128() * price_cents.as_u128() / Sats::ONE_BTC_U128)
        }
    }
}
