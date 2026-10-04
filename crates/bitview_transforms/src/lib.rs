//! Stateless, element-wise Bitcoin value transforms.
mod arithmetic;
mod currency;
mod ohlc;
mod ratio;

pub use arithmetic::{
    BlockCountTarget, BlocksToDays, CountPerSecond, DaysToYears, DifficultyToHashrate, MaskSats,
    OneMinusPpm, ReturnTenths, ThsToPhs, TimesSqrt, WeightToVSize,
};
pub use currency::{
    AvgCentsToUsd, AvgSatsToBtc, CentsSignedToDollars, CentsTimesTenths, CentsUnsignedToDollars,
    CentsUnsignedToSats, DollarsToSatsFract, SatsSignedToBitcoin, SatsToBitcoin, SatsToCents,
};
pub use ohlc::{OhlcCentsToDollars, OhlcCentsToHighCents, OhlcCentsToLowCents, OhlcCentsToSats};
pub use ratio::{
    BoundedOdds, BoundedToRatio, Cagr, FixedToPercent, FixedToRatio, MvrvToNupl, PriceTimesRatio,
    RatioBytes, RatioCents, RatioCentsOrOne, RatioCentsSignedCents, RatioCount, RatioDiffCents,
    RatioDiffDollars, RatioDiffFloat32, RatioDollars, RatioSats, price_ratio,
};
