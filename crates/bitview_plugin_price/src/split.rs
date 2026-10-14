use bitview_primitives::OHLCCents;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyIndexes, Price, ops::Convert};
use brk_types::{Cents, Dollars, Version};
use vecdb::UnaryTransform;

use crate::ohlc::OhlcPrice;

/// One price of an OHLC candle.
macro_rules! ohlc_field {
    ($($name:ident => $field:ident),+ $(,)?) => {$(
        struct $name;

        impl UnaryTransform<OHLCCents, Cents> for $name {
            #[inline(always)]
            fn apply(cents: OHLCCents) -> Cents {
                *cents.$field
            }
        }
    )+};
}

ohlc_field!(
    OhlcCentsToOpenCents => open,
    OhlcCentsToHighCents => high,
    OhlcCentsToLowCents => low,
    OhlcCentsToCloseCents => close,
);

type IndexedPrice = Price<LazyIndexes<Cents, OHLCCents>, LazyIndexes<Dollars, Cents>>;

/// Open, high, low and close fields.
#[derive(Clone, Traversable)]
pub(crate) struct Ohlc<T> {
    /// Opening price for each supported period. A populated period uses its
    /// first block-level price; an empty period carries the previous close.
    open: T,
    /// Highest price for each supported period. A populated period uses its
    /// maximum block-level price; an empty period carries the previous close.
    high: T,
    /// Lowest price for each supported period. A populated period uses its
    /// minimum block-level price; an empty period carries the previous close.
    low: T,
    /// Closing price for each supported period. A populated period uses its
    /// final block-level price; an empty period carries the previous close.
    close: T,
}

/// Each price of the OHLC candle as its own series: an empty period is a flat candle at the
/// previous close.
#[derive(Clone, Traversable)]
pub(crate) struct SplitPrice(Ohlc<IndexedPrice>);

impl SplitPrice {
    pub(crate) fn new(name: &str, version: Version, ohlc: &OhlcPrice) -> Self {
        Self(Ohlc {
            open: split::<OhlcCentsToOpenCents>(name, "open", version, ohlc),
            high: split::<OhlcCentsToHighCents>(name, "high", version, ohlc),
            low: split::<OhlcCentsToLowCents>(name, "low", version, ohlc),
            close: split::<OhlcCentsToCloseCents>(name, "close", version, ohlc),
        })
    }
}

/// One price of the candle, `F` picking it.
fn split<F: UnaryTransform<OHLCCents, Cents>>(
    name: &str,
    field: &str,
    version: Version,
    ohlc: &OhlcPrice,
) -> IndexedPrice {
    let cents = ohlc
        .cents
        .transformed::<Cents, F>(&format!("{name}_{field}_cents"), version);
    let usd =
        LazyIndexes::from_lazy_indexes::<Convert, _>(&format!("{name}_{field}"), version, &cents);
    Price { usd, cents }
}
