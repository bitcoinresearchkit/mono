//! Stateless, element-wise Bitcoin value transforms.
mod arithmetic;
mod convert;
mod currency;
mod ohlc;
mod ratio;

pub use arithmetic::{
    BlockCountTarget, BlocksToDays, CountPerSecond, DaysToYears, DifficultyToHashrate, MaskSats,
    OneMinusPpm, ReturnTenths, ThsToPhs, TimesSqrt, WeightToVSize,
};
pub use convert::Convert;
pub use currency::{
    AvgCentsToUsd, AvgSatsToBtc, CentsTimesTenths, CentsUnsignedToSats, SatsToCents,
};
pub use ohlc::{OhlcCentsToHighCents, OhlcCentsToLowCents};
pub use ratio::{
    BoundedOdds, BoundedToRatio, Cagr, FixedToPercent, FixedToRatio, MvrvToNupl, PriceTimesRatio,
    Quotient, RatioCentsOrOne, RatioDiffFloat32, RatioDollars, RelativeChange, price_ratio,
};
