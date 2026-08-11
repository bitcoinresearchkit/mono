use brk_types::{
    Bitcoin, Cents, CentsSigned, Dollars, Sats, SatsFract, SatsSigned, StoredF32, StoredU64,
};
use vecdb::{BinaryTransform, UnaryTransform, unlikely};

pub struct SatsToBitcoin;

impl UnaryTransform<Sats, Bitcoin> for SatsToBitcoin {
    #[inline(always)]
    fn apply(sats: Sats) -> Bitcoin {
        Bitcoin::from(sats)
    }
}

pub struct StoredU64ToSats;

impl UnaryTransform<StoredU64, Sats> for StoredU64ToSats {
    #[inline(always)]
    fn apply(value: StoredU64) -> Sats {
        Sats::new(value.into())
    }
}

pub struct StoredU64ToCents;

impl UnaryTransform<StoredU64, Cents> for StoredU64ToCents {
    #[inline(always)]
    fn apply(value: StoredU64) -> Cents {
        Cents::new(value.into())
    }
}

pub struct SatsSignedToBitcoin;

impl UnaryTransform<SatsSigned, Bitcoin> for SatsSignedToBitcoin {
    #[inline(always)]
    fn apply(sats: SatsSigned) -> Bitcoin {
        Bitcoin::from(sats)
    }
}

pub struct AvgSatsToBtc;

impl UnaryTransform<StoredF32, Bitcoin> for AvgSatsToBtc {
    #[inline(always)]
    fn apply(sats: StoredF32) -> Bitcoin {
        Bitcoin::from(f64::from(sats) / Sats::ONE_BTC_U128 as f64)
    }
}

pub struct AvgCentsToUsd;

impl UnaryTransform<StoredF32, Dollars> for AvgCentsToUsd {
    #[inline(always)]
    fn apply(cents: StoredF32) -> Dollars {
        Dollars::from(f64::from(cents) / 100.0)
    }
}

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

pub struct CentsUnsignedToDollars;

impl UnaryTransform<Cents, Dollars> for CentsUnsignedToDollars {
    #[inline(always)]
    fn apply(cents: Cents) -> Dollars {
        cents.into()
    }
}

pub struct NegCentsUnsignedToDollars;

impl UnaryTransform<Cents, Dollars> for NegCentsUnsignedToDollars {
    #[inline(always)]
    fn apply(cents: Cents) -> Dollars {
        -Dollars::from(cents)
    }
}

pub struct CentsSignedToDollars;

impl UnaryTransform<CentsSigned, Dollars> for CentsSignedToDollars {
    #[inline(always)]
    fn apply(cents: CentsSigned) -> Dollars {
        cents.into()
    }
}

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

pub struct CentsTimesTenths<const V: u16>;

impl<const V: u16> UnaryTransform<Cents, Cents> for CentsTimesTenths<V> {
    #[inline(always)]
    fn apply(c: Cents) -> Cents {
        if unlikely(c.is_nan()) {
            Cents::NAN
        } else {
            Cents::from(c.as_u128() * V as u128 / 10)
        }
    }
}

pub struct DollarsToSatsFract;

impl UnaryTransform<Dollars, SatsFract> for DollarsToSatsFract {
    #[inline(always)]
    fn apply(usd: Dollars) -> SatsFract {
        SatsFract::ONE_BTC / usd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cents_outputs_propagate_nan() {
        assert!(SatsToCents::apply(Sats::ONE_BTC, Cents::NAN).is_nan());
        assert!(CentsTimesTenths::<24>::apply(Cents::NAN).is_nan());
    }

    #[test]
    #[should_panic(expected = "Cents::NAN cannot be converted to whole Sats")]
    fn whole_sats_reject_nan() {
        CentsUnsignedToSats::apply(Cents::NAN);
    }
}
