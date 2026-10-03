use bitview_primitives::{OHLCCents, OHLCDollars, OHLCSats};
use bitview_transforms::{OhlcCentsToDollars, OhlcCentsToSats};
use brk_types::Version;

use crate::{IndexSources, LazyIndexes, LazyOhlcCentsVecs, Price, SpotPrice};

pub type OhlcPrice =
    Price<LazyOhlcCentsVecs, LazyIndexes<OHLCDollars, OHLCCents>, LazyIndexes<OHLCSats, OHLCCents>>;

impl OhlcPrice {
    pub fn from_spot(
        name: &str,
        version: Version,
        indexes: &IndexSources,
        spot: &SpotPrice,
    ) -> Self {
        let cents = LazyOhlcCentsVecs::new(
            &format!("{name}_cents"),
            version,
            indexes,
            spot.cents.height.read_only_boxed_clone(),
        );
        let usd = LazyIndexes::from_ohlc_indexes::<OhlcCentsToDollars>(name, version, &cents);
        // The reciprocal candle transform swaps high and low internally.
        let sats = LazyIndexes::from_ohlc_indexes::<OhlcCentsToSats>(
            &format!("{name}_sats"),
            version,
            &cents,
        );
        Self { usd, cents, sats }
    }
}
use vecdb::ReadableCloneableVec;
