//! Stateless, element-wise Bitcoin value transforms.
mod arithmetic;
mod currency;
mod ohlc;
mod ratio;

pub use arithmetic::{
    BlockCountTarget, BlocksToDaysF32, DaysToYears, DifficultyToHashF64, MaskSats, OneMinusPpm,
    PerSecond, ReturnTenths, ThsToPhsF32, TimesSqrt, WeightToVSize,
};
pub use currency::{
    AvgCentsToUsd, AvgSatsToBtc, CentsSignedToDollars, CentsTimesTenths, CentsUnsignedToDollars,
    CentsUnsignedToSats, DollarsToSatsFract, SatsSignedToBitcoin, SatsToBitcoin, SatsToCents,
};
pub use ohlc::{OhlcCentsToDollars, OhlcCentsToHighCents, OhlcCentsToLowCents, OhlcCentsToSats};
pub use ratio::{
    BoundedOdds, BoundedToRatio, Cagr, FixedToPercent, FixedToRatio, MvrvToNupl, PriceTimesRatio,
    RatioBytes, RatioCents, RatioCentsOrOne, RatioCentsSignedCents, RatioCount, RatioDiffCents,
    RatioDiffDollars, RatioDiffF32, RatioDollars, RatioSats, price_ratio,
};
