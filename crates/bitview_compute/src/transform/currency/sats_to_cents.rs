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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_product_matches_u128_reference() {
        for index in 0..1_200_000_u64 {
            let sats =
                Sats::from(index.wrapping_mul(6_364_136_223_846_793_005) % 2_100_000_000_000_001);
            let cents = Cents::from(index.wrapping_mul(2_654_435_761) % 20_000_001);
            let expected = Cents::from(sats.as_u128() * cents.as_u128() / Sats::ONE_BTC_U128);
            assert_eq!(SatsToCents::apply(sats, cents), expected);
        }
    }
}
