use bitview_primitives::OHLCCents;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyIndexes, Price};
use brk_types::{Cents, Dollars, Height, Version};
use vecdb::{Convert, ReadableBoxedVec, UnaryTransform};

use crate::{
    endpoint::{EndpointPrice, endpoint_price},
    ohlc::OhlcPrice,
};

macro_rules! ohlc_field {
    ($($name:ident => $field:ident),+ $(,)?) => {$(
        struct $name;

        impl UnaryTransform<OHLCCents, Cents> for $name {
            #[inline(always)]
            fn apply(cents: OHLCCents) -> Cents {
                cents.$field
            }
        }
    )+};
}

ohlc_field!(
    OhlcCentsToHighCents => high,
    OhlcCentsToLowCents => low,
);

type ExtremePrice = Price<LazyIndexes<Cents, OHLCCents>, LazyIndexes<Dollars, Cents>>;

/// Each price of the OHLC candle as its own series: an empty period is a flat candle at the
/// previous close. Open and close read one height per period; high and low need the candle.
#[derive(Clone, Traversable)]
pub(crate) struct SplitPrice {
    /// Opening price for each supported period. A populated period uses its
    /// first priced block-level price (zero before the first price, block
    /// 74,641); an empty period carries the previous close.
    open: EndpointPrice<false>,
    /// Highest price for each supported period. A populated period uses its
    /// maximum block-level price; an empty period carries the previous close.
    high: ExtremePrice,
    /// Lowest price for each supported period. A populated period uses its
    /// minimum priced block-level price (zero before the first price, block
    /// 74,641); an empty period carries the previous close.
    low: ExtremePrice,
    /// Closing price for each supported period. A populated period uses its
    /// final block-level price; an empty period carries the previous close.
    close: EndpointPrice<true>,
}

impl SplitPrice {
    pub(crate) fn new(
        name: &str,
        version: Version,
        indexes: &IndexSources,
        prices: &ReadableBoxedVec<Height, Cents>,
        ohlc: &OhlcPrice,
    ) -> Self {
        Self {
            open: endpoint_price::<false>(&format!("{name}_open"), version, indexes, prices),
            high: extreme::<OhlcCentsToHighCents>(name, "high", version, ohlc),
            low: extreme::<OhlcCentsToLowCents>(name, "low", version, ohlc),
            close: endpoint_price::<true>(&format!("{name}_close"), version, indexes, prices),
        }
    }
}

fn extreme<F: UnaryTransform<OHLCCents, Cents>>(
    name: &str,
    field: &str,
    version: Version,
    ohlc: &OhlcPrice,
) -> ExtremePrice {
    let cents =
        LazyIndexes::from_resolutions::<F>(&format!("{name}_{field}_cents"), version, &ohlc.cents);
    let usd =
        LazyIndexes::from_resolutions::<Convert>(&format!("{name}_{field}"), version, &cents.0);
    Price { usd, cents }
}
