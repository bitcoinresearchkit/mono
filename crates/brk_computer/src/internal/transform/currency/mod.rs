mod avg_cents_to_usd;
mod avg_sats_to_btc;
mod cents_signed_to_dollars;
mod cents_times_tenths;
mod cents_unsigned_to_dollars;
mod cents_unsigned_to_sats;
mod dollars_to_sats_fract;
mod neg_cents_unsigned_to_dollars;
mod sats_signed_to_bitcoin;
mod sats_to_bitcoin;
mod sats_to_cents;
mod stored_u64_to_cents;
mod stored_u64_to_sats;

pub use avg_cents_to_usd::AvgCentsToUsd;
pub use avg_sats_to_btc::AvgSatsToBtc;
pub use cents_signed_to_dollars::CentsSignedToDollars;
pub use cents_times_tenths::CentsTimesTenths;
pub use cents_unsigned_to_dollars::CentsUnsignedToDollars;
pub use cents_unsigned_to_sats::CentsUnsignedToSats;
pub use dollars_to_sats_fract::DollarsToSatsFract;
pub use neg_cents_unsigned_to_dollars::NegCentsUnsignedToDollars;
pub use sats_signed_to_bitcoin::SatsSignedToBitcoin;
pub use sats_to_bitcoin::SatsToBitcoin;
pub use sats_to_cents::SatsToCents;
pub use stored_u64_to_cents::StoredU64ToCents;
pub use stored_u64_to_sats::StoredU64ToSats;

#[cfg(test)]
mod tests {
    use brk_types::{Cents, Sats};
    use vecdb::{BinaryTransform, UnaryTransform};

    use super::{CentsTimesTenths, CentsUnsignedToSats, SatsToCents};

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
