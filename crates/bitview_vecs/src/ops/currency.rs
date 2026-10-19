use brk_types::{Cents, Sats};
use vecdb::UnaryTransform;

/// A price (cents per BTC) to the whole sats one dollar buys.
pub struct CentsToSatsPerDollar;

impl UnaryTransform<Cents, Sats> for CentsToSatsPerDollar {
    #[inline(always)]
    fn apply(cents: Cents) -> Sats {
        cents.sats_per_dollar()
    }
}
