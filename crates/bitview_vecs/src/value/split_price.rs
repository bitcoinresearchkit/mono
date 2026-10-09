use bitview_collections::Ohlc;
use bitview_primitives::OHLCCents;
use bitview_transforms::{Convert, OhlcCentsToHighCents, OhlcCentsToLowCents};
use bitview_traversable::Traversable;
use brk_types::{Cents, Dollars, Version};
use derive_more::{Deref, DerefMut};

use crate::{IndexSources, LazyIndexes, OhlcPrice, Price, Resolutions, SpotPrice};

pub type IndexedPrice<S = OHLCCents> = Price<LazyIndexes<Cents, S>, LazyIndexes<Dollars, Cents>>;
pub type ClosePrice = Price<Resolutions<Cents>, Resolutions<Dollars>>;

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct SplitPrice(pub Ohlc<IndexedPrice, ClosePrice, IndexedPrice<Cents>>);

impl SplitPrice {
    pub fn new(
        name: &str,
        version: Version,
        indexes: &IndexSources,
        spot: &SpotPrice,
        ohlc: &OhlcPrice,
    ) -> Self {
        let open_cents = LazyIndexes::from_open_source(
            &format!("{name}_open_cents"),
            version,
            &spot.cents.height,
            indexes,
        );
        let high_cents = LazyIndexes::from_ohlc_indexes::<OhlcCentsToHighCents>(
            &format!("{name}_high_cents"),
            version,
            &ohlc.cents,
        );
        let low_cents = LazyIndexes::from_ohlc_indexes::<OhlcCentsToLowCents>(
            &format!("{name}_low_cents"),
            version,
            &ohlc.cents,
        );
        let open_usd = LazyIndexes::from_lazy_indexes::<Convert, _>(
            &format!("{name}_open"),
            version,
            &open_cents,
        );
        let high_usd = LazyIndexes::from_lazy_indexes::<Convert, _>(
            &format!("{name}_high"),
            version,
            &high_cents,
        );
        let low_usd = LazyIndexes::from_lazy_indexes::<Convert, _>(
            &format!("{name}_low"),
            version,
            &low_cents,
        );
        Self(Ohlc {
            open: Price {
                usd: open_usd,
                cents: open_cents,
            },
            high: Price {
                usd: high_usd,
                cents: high_cents,
            },
            low: Price {
                usd: low_usd,
                cents: low_cents,
            },
            close: Price {
                usd: Resolutions::from_source(
                    &format!("{name}_close"),
                    &spot.usd.height,
                    version,
                    indexes,
                ),
                cents: Resolutions::from_source(
                    &format!("{name}_close_cents"),
                    &spot.cents.height,
                    version,
                    indexes,
                ),
            },
        })
    }
}
