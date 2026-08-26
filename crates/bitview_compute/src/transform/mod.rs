mod arithmetic;
mod cagr;
mod currency;
mod days_to_years;
mod fixed_ratio;
mod price_times_ratio;
mod ratio;
mod ratio_cents_f32;
mod sopr_ratio;
mod specialized;
mod times_sqrt;

pub use arithmetic::{
    BlocksToDaysF32, DifficultyToHashF64, HalveCents, HalveDollars, HalveSats, HalveSatsToBitcoin,
    Identity, MaskSats, OddsF64, OneMinusF64, OneMinusPpm, PerSecond, ReturnF32Tenths, ReturnI8,
    ReturnU16, StoredU16ToStoredU64, StoredU64ToStoredU32, ThsToPhsF32, VBytesToWeight,
    WeightToVSize,
};
pub use cagr::Cagr;
pub use currency::{
    AvgCentsToUsd, AvgSatsToBtc, CentsSignedToDollars, CentsTimesTenths, CentsUnsignedToDollars,
    CentsUnsignedToSats, DollarsToSatsFract, NegCentsUnsignedToDollars, SatsSignedToBitcoin,
    SatsToBitcoin, SatsToCents, StoredU64ToCents, StoredU64ToSats,
};
pub use days_to_years::DaysToYears;
pub use fixed_ratio::{FixedToPercent, FixedToRatio};
pub use price_times_ratio::PriceTimesRatio;
pub use ratio::{
    RatioBytes, RatioCents, RatioCentsSignedCents, RatioDiffCents, RatioDiffDollars, RatioDiffF32,
    RatioDollars, RatioSats, RatioU64,
};
pub use ratio_cents_f32::*;
pub use sopr_ratio::SoprRatio;
pub use specialized::{
    BlockCountTarget1m, BlockCountTarget1w, BlockCountTarget1y, BlockCountTarget24h,
    OhlcCentsToDollars, OhlcCentsToHighCents, OhlcCentsToLowCents, OhlcCentsToOpenCents,
    OhlcCentsToSats,
};
pub use times_sqrt::TimesSqrt;
