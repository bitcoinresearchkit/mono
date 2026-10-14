use bitview_collections::Ohlc;
use bitview_primitives::OHLCCents;
use bitview_transforms::{
    Convert, OhlcCentsToCloseCents, OhlcCentsToHighCents, OhlcCentsToLowCents, OhlcCentsToOpenCents,
};
use bitview_traversable::Traversable;
use brk_types::{Cents, Dollars, Version};
use derive_more::{Deref, DerefMut};
use vecdb::UnaryTransform;

use crate::{LazyIndexes, OhlcPrice, Price};

pub type IndexedPrice = Price<LazyIndexes<Cents, OHLCCents>, LazyIndexes<Dollars, Cents>>;

/// Each price of the OHLC candle as its own series: an empty period is a flat candle at the
/// previous close.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct SplitPrice(pub Ohlc<IndexedPrice>);

impl SplitPrice {
    pub fn new(name: &str, version: Version, ohlc: &OhlcPrice) -> Self {
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
    let cents =
        LazyIndexes::from_ohlc_indexes::<F>(&format!("{name}_{field}_cents"), version, &ohlc.cents);
    let usd =
        LazyIndexes::from_lazy_indexes::<Convert, _>(&format!("{name}_{field}"), version, &cents);
    Price { usd, cents }
}
