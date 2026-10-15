mod candle;
mod resolutions;

use bitview_primitives::{OHLCCents, OHLCDollars};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyIndexes, Price};
use brk_types::Version;
use derive_more::Deref;
use vecdb::{Convert, ReadableCloneableVec};

use crate::spot::SpotPrice;
use resolutions::LazyOhlcCentsVecs;

/// OHLC candles of the spot price per resolution: cents, with a lazy USD view.
#[derive(Clone, Deref, Traversable)]
pub(crate) struct OhlcPrice(Price<LazyOhlcCentsVecs, LazyIndexes<OHLCDollars, OHLCCents>>);

impl OhlcPrice {
    pub(crate) fn from_spot(
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
        let usd = cents.transformed::<OHLCDollars, Convert>(name, version);
        Self(Price { usd, cents })
    }
}
