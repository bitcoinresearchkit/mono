use bitview_primitives::{OHLCCents, OHLCDollars};
use bitview_transforms::Convert;
use brk_types::Version;

use crate::{IndexSources, LazyIndexes, LazyOhlcCentsVecs, Price, SpotPrice};

pub type OhlcPrice = Price<LazyOhlcCentsVecs, LazyIndexes<OHLCDollars, OHLCCents>>;

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
        let usd = LazyIndexes::from_ohlc_indexes::<Convert>(name, version, &cents);
        Self { usd, cents }
    }
}
use vecdb::ReadableCloneableVec;
