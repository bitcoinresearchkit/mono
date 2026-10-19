mod candle;

use bitview_primitives::{OHLCCents, OHLCDollars};
use bitview_traversable::Traversable;
use bitview_vecs::{AggResolutions, IndexSources, LazyIndexes, Price};
use brk_types::{Cents, Height, Version};
use derive_more::Deref;
use vecdb::{Convert, ReadableBoxedVec};

use candle::Ohlc;

const COMPUTE_VERSION: Version = Version::new(3);

pub(crate) type OhlcCents = AggResolutions<OHLCCents, Cents, Ohlc>;

/// OHLC candles of the spot price per resolution: cents, with a lazy USD view.
#[derive(Clone, Deref, Traversable)]
pub(crate) struct OhlcPrice(Price<OhlcCents, LazyIndexes<OHLCDollars, OHLCCents>>);

impl OhlcPrice {
    pub(crate) fn new(
        name: &str,
        version: Version,
        indexes: &IndexSources,
        prices: &ReadableBoxedVec<Height, Cents>,
    ) -> Self {
        let cents = OhlcCents::aggregate(
            &format!("{name}_cents"),
            version + COMPUTE_VERSION,
            prices,
            indexes,
        );
        let usd = LazyIndexes::from_resolutions::<Convert>(name, version, &cents);
        Self(Price { usd, cents })
    }
}
