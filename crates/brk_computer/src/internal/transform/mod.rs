mod arithmetic;
mod currency;
mod derived;
mod fixed_ratio;
mod ratio;
mod sopr_ratio;
mod specialized;

pub use arithmetic::{
    BlocksToDaysF32, DifficultyToHashF64, HalveCents, HalveDollars, HalveSats, HalveSatsToBitcoin,
    Identity, MaskSats, OddsF64, OneMinusF64, OneMinusPpm, PerSecond, ReturnF32Tenths, ReturnI8,
    ReturnU16, StoredU16ToStoredU64, StoredU64ToStoredU32, ThsToPhsF32, VBytesToWeight,
    WeightToVSize,
};
pub use currency::{
    AvgCentsToUsd, AvgSatsToBtc, CentsSignedToDollars, CentsTimesTenths, CentsUnsignedToDollars,
    CentsUnsignedToSats, DollarsToSatsFract, NegCentsUnsignedToDollars, SatsSignedToBitcoin,
    SatsToBitcoin, SatsToCents, StoredU64ToCents, StoredU64ToSats,
};
pub use derived::{
    Cagr, Days1, Days7, Days30, Days365, DaysToYears, PriceTimesRatio, RatioCents64, TimesSqrt,
};
pub use fixed_ratio::{FixedToPercent, FixedToRatio};
pub use ratio::{
    RatioBytes, RatioCents, RatioCentsSignedCents, RatioDiffCents, RatioDiffDollars, RatioDiffF32,
    RatioDollars, RatioSats, RatioU64,
};
pub use sopr_ratio::SoprRatio;
pub use specialized::{
    BlockCountTarget1m, BlockCountTarget1w, BlockCountTarget1y, BlockCountTarget24h,
    OhlcCentsToDollars, OhlcCentsToHighCents, OhlcCentsToLowCents, OhlcCentsToOpenCents,
    OhlcCentsToSats,
};
