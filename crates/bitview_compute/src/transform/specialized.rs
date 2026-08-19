use brk_types::{
    Cents, Close, Day1, Day3, Epoch, Halving, Height, High, Hour1, Hour4, Hour12, Low, Minute10,
    Minute30, Month1, Month3, Month6, OHLCCents, OHLCDollars, OHLCSats, Open, StoredU64, Week1,
    Year1, Year10,
};
use vecdb::UnaryTransform;

use super::CentsUnsignedToSats;
use crate::{
    TARGET_BLOCKS_PER_DAY, TARGET_BLOCKS_PER_MONTH, TARGET_BLOCKS_PER_WEEK, TARGET_BLOCKS_PER_YEAR,
};

macro_rules! const_block_target {
    ($name:ident, $value:expr) => {
        pub struct $name;
        const_block_target!(@impl $name, $value, Height, Minute10, Minute30, Hour1, Hour4, Hour12, Day1, Day3, Week1, Month1, Month3, Month6, Year1, Year10, Halving, Epoch);
    };
    (@impl $name:ident, $value:expr, $($idx:ty),*) => {
        $(
            impl UnaryTransform<$idx, StoredU64> for $name {
                #[inline(always)]
                fn apply(_: $idx) -> StoredU64 {
                    StoredU64::from($value)
                }
            }
        )*
    };
}

const_block_target!(BlockCountTarget24h, TARGET_BLOCKS_PER_DAY);
const_block_target!(BlockCountTarget1w, TARGET_BLOCKS_PER_WEEK);
const_block_target!(BlockCountTarget1m, TARGET_BLOCKS_PER_MONTH);
const_block_target!(BlockCountTarget1y, TARGET_BLOCKS_PER_YEAR);

pub struct OhlcCentsToDollars;

impl UnaryTransform<OHLCCents, OHLCDollars> for OhlcCentsToDollars {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> OHLCDollars {
        OHLCDollars::from(cents)
    }
}

pub struct OhlcCentsToOpenCents;

impl UnaryTransform<OHLCCents, Cents> for OhlcCentsToOpenCents {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> Cents {
        *cents.open
    }
}

pub struct OhlcCentsToHighCents;

impl UnaryTransform<OHLCCents, Cents> for OhlcCentsToHighCents {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> Cents {
        *cents.high
    }
}

pub struct OhlcCentsToLowCents;

impl UnaryTransform<OHLCCents, Cents> for OhlcCentsToLowCents {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> Cents {
        *cents.low
    }
}

pub struct OhlcCentsToSats;

impl UnaryTransform<OHLCCents, OHLCSats> for OhlcCentsToSats {
    #[inline(always)]
    fn apply(cents: OHLCCents) -> OHLCSats {
        OHLCSats {
            open: Open::new(CentsUnsignedToSats::apply(*cents.open)),
            high: High::new(CentsUnsignedToSats::apply(*cents.low)),
            low: Low::new(CentsUnsignedToSats::apply(*cents.high)),
            close: Close::new(CentsUnsignedToSats::apply(*cents.close)),
        }
    }
}
