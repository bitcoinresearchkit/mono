use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Version};
use vecdb::{CachedBoxedVec, CachedReadableVec, CachedVec, ReadableCloneableVec};

use bitview_compute::{CentsTimesTenths, LazyPerBlock, LazyPriceWithRatioPerBlock, Price};

use super::lazy_sma::{LazySmaVec, SmaPrefixSumVec};

#[derive(Clone, Traversable)]
pub struct SmaVecs {
    pub _1w: LazyPriceWithRatioPerBlock,
    pub _8d: LazyPriceWithRatioPerBlock,
    pub _13d: LazyPriceWithRatioPerBlock,
    pub _21d: LazyPriceWithRatioPerBlock,
    pub _1m: LazyPriceWithRatioPerBlock,
    pub _34d: LazyPriceWithRatioPerBlock,
    pub _50d: LazyPriceWithRatioPerBlock,
    pub _55d: LazyPriceWithRatioPerBlock,
    pub _89d: LazyPriceWithRatioPerBlock,
    pub _111d: LazyPriceWithRatioPerBlock,
    pub _144d: LazyPriceWithRatioPerBlock,
    pub _200d: LazyPriceWithRatioPerBlock,
    pub _350d: LazyPriceWithRatioPerBlock,
    pub _1y: LazyPriceWithRatioPerBlock,
    pub _2y: LazyPriceWithRatioPerBlock,
    pub _200w: LazyPriceWithRatioPerBlock,
    pub _4y: LazyPriceWithRatioPerBlock,
    /// The 200-day simple moving average multiplied by 2.4.
    #[traversable(wrap = "200d", rename = "x2_4")]
    pub _200d_x2_4: Price<LazyPerBlock<Cents, Cents>>,
    /// The 200-day simple moving average multiplied by 0.8.
    #[traversable(wrap = "200d", rename = "x0_8")]
    pub _200d_x0_8: Price<LazyPerBlock<Cents, Cents>>,
    /// The 350-day simple moving average multiplied by two.
    #[traversable(wrap = "350d", rename = "x2")]
    pub _350d_x2: Price<LazyPerBlock<Cents, Cents>>,
}

const VERSION: Version = Version::ONE;

impl SmaVecs {
    pub fn new(
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        lookback: &bitview_plugin_blocks::LookbackVecs,
        spot_price: CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let version = version + VERSION;
        let prefix_sum = CachedVec::wrap(SmaPrefixSumVec::new(
            "price_sma_prefix_sum",
            version,
            spot_price.clone(),
        ));

        macro_rules! sma {
            ($name:literal, $days:expr) => {
                LazyPriceWithRatioPerBlock::from_height_source(
                    concat!("price_sma_", $name),
                    version,
                    LazySmaVec::new(
                        concat!("price_sma_", $name, "_cents_source"),
                        version,
                        lookback.start_vec($days).read_only_boxed_clone(),
                        prefix_sum.cached_boxed_clone(),
                    ),
                    indexes,
                    &spot_price,
                )
            };
        }

        let _200d = sma!("200d", 200);
        let _350d = sma!("350d", 350);

        let _200d_x2_4 = Price::from_lazy_cents_source::<CentsTimesTenths<24>, _>(
            "price_sma_200d_x2_4",
            version,
            &_200d.cents,
        );
        let _200d_x0_8 = Price::from_lazy_cents_source::<CentsTimesTenths<8>, _>(
            "price_sma_200d_x0_8",
            version,
            &_200d.cents,
        );
        let _350d_x2 = Price::from_lazy_cents_source::<CentsTimesTenths<20>, _>(
            "price_sma_350d_x2",
            version,
            &_350d.cents,
        );

        Self {
            _1w: sma!("1w", 7),
            _8d: sma!("8d", 8),
            _13d: sma!("13d", 13),
            _21d: sma!("21d", 21),
            _1m: sma!("1m", 30),
            _34d: sma!("34d", 34),
            _50d: sma!("50d", 50),
            _55d: sma!("55d", 55),
            _89d: sma!("89d", 89),
            _111d: sma!("111d", 111),
            _144d: sma!("144d", 144),
            _200d,
            _350d,
            _1y: sma!("1y", 365),
            _2y: sma!("2y", 2 * 365),
            _200w: sma!("200w", 200 * 7),
            _4y: sma!("4y", 4 * 365),
            _200d_x2_4,
            _200d_x0_8,
            _350d_x2,
        }
    }
}
