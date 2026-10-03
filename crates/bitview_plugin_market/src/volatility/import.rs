use bitview_collections::Windows;
use bitview_transforms::TimesSqrt;
use bitview_vecs::LazyPerBlock;
use brk_types::Version;

use super::{super::returns, Vecs};

impl Vecs {
    pub(crate) fn new(version: Version, returns: &returns::Vecs) -> Self {
        let v2 = Version::TWO;

        let _24h = LazyPerBlock::from_resolutions::<TimesSqrt<1>>(
            "price_volatility_24h",
            version + v2,
            &returns.sd_24h._24h.sd,
        );

        let _1w = LazyPerBlock::from_resolutions::<TimesSqrt<7>>(
            "price_volatility_1w",
            version + v2,
            &returns.sd_24h._1w.sd,
        );

        let _1m = LazyPerBlock::from_resolutions::<TimesSqrt<30>>(
            "price_volatility_1m",
            version + v2,
            &returns.sd_24h._1m.sd,
        );

        let _1y = LazyPerBlock::from_resolutions::<TimesSqrt<365>>(
            "price_volatility_1y",
            version + v2,
            &returns.sd_24h._1y.sd,
        );

        Self(Windows {
            _24h,
            _1w,
            _1m,
            _1y,
        })
    }
}
