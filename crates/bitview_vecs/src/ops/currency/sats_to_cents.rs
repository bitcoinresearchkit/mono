use brk_types::{Cents, CentsSats, Sats};
use vecdb::{BinaryTransform, unlikely};

/// A satoshi amount's value at a price, rounded to the nearest cent.
pub struct SatsToCents;

impl BinaryTransform<Sats, Cents, Cents> for SatsToCents {
    #[inline(always)]
    fn apply(sats: Sats, price_cents: Cents) -> Cents {
        if unlikely(price_cents.is_nan()) {
            Cents::NAN
        } else if let Some(value) = u64::from(sats)
            .checked_mul(u64::from(price_cents))
            .and_then(|value| value.checked_add(Sats::ONE_BTC_U64 / 2))
        {
            Cents::from(value / Sats::ONE_BTC_U64)
        } else {
            CentsSats::from_price_sats(price_cents, sats).to_cents()
        }
    }
}
