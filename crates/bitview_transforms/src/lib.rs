//! Stateless, element-wise Bitcoin value transforms.
mod arithmetic;
mod currency;
mod ohlc;
mod ratio;

pub use arithmetic::{
    BlockCountTarget, BlocksToDaysF32, DaysToYears, DifficultyToHashF64, MaskSats, OneMinusPpm,
    PerSecond, ReturnF32Tenths, ReturnI8, ReturnU16, StoredU64ToStoredU32, ThsToPhsF32, TimesSqrt,
    VBytesToWeight, WeightToVSize,
};
pub use currency::{
    AvgCentsToUsd, AvgSatsToBtc, CentsSignedToDollars, CentsTimesTenths, CentsUnsignedToDollars,
    CentsUnsignedToSats, DollarsToSatsFract, SatsSignedToBitcoin, SatsToBitcoin, SatsToCents,
    StoredU64ToCents, StoredU64ToSats,
};
pub use ohlc::{OhlcCentsToDollars, OhlcCentsToHighCents, OhlcCentsToLowCents, OhlcCentsToSats};
pub use ratio::{
    BoundedOddsF64, BoundedToF64, Cagr, FixedToPercent, FixedToRatio, MvrvToNupl, PriceTimesRatio,
    RatioBytes, RatioCents, RatioCentsF32, RatioCentsSignedCents, RatioDiffCents, RatioDiffDollars,
    RatioDiffF32, RatioDollars, RatioSats, RatioU64, SoprRatio, price_ratio,
};
