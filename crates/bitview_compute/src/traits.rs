use std::ops::{Add, AddAssign};

use bitview_primitives::{
    Bytes, Bytes32, BytesFract, CentsFract, CoinBlocks, CoinDays, Count, Count16, Count32,
    CountFract, DollarBlocks, DollarDaysPerCoin, Float64, PartsPerMillion32, PartsPerMillion64,
    PartsPerMillionSigned32, PartsPerMillionSigned64, PriceRatio, SatsFract, Seconds, SecondsFract,
    SigOpsFract, VSizeFract, WeightFract,
};
use brk_types::{Cents, CentsSigned, Sats, SigOps, VSize, Weight, Weight32};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{CheckedSub, Formattable, PcoVecValue};

pub trait ComputedVecValue
where
    Self:
        PcoVecValue + From<usize> + Add<Output = Self> + AddAssign + Ord + Formattable + Serialize,
{
}
impl<T> ComputedVecValue for T where
    T: PcoVecValue + From<usize> + Add<Output = Self> + AddAssign + Ord + Formattable + Serialize
{
}

pub trait NumericValue: ComputedVecValue + CheckedSub + Default + From<f64> + Into<f64> {}

impl<T> NumericValue for T where T: ComputedVecValue + CheckedSub + Default + From<f64> + Into<f64> {}

/// A quantity: how it accumulates and how its means are typed, keeping its unit.
pub trait Quantity: Sized {
    /// A mean of the quantity (a mean count of 1.5 is representable).
    type Fract: NumericValue + JsonSchema;
    /// Sums of the quantity (cumulatives, rolling sums): a wide enough type of the same unit.
    type Sum: NumericValue + JsonSchema + Quantity + From<Self>;
}

macro_rules! quantities {
    ($($quantity:ty => ($fract:ty, $sum:ty),)*) => {
        $(impl Quantity for $quantity {
            type Fract = $fract;
            type Sum = $sum;
        })*
    };
}

quantities! {
    Count => (CountFract, Count),
    Count16 => (CountFract, Count),
    Count32 => (CountFract, Count),
    Bytes => (BytesFract, Bytes),
    Bytes32 => (BytesFract, Bytes),
    VSize => (VSizeFract, VSize),
    Weight => (WeightFract, Weight),
    Weight32 => (WeightFract, Weight),
    SigOps => (SigOpsFract, SigOps),
    Seconds => (SecondsFract, Seconds),
    Sats => (SatsFract, Sats),
    Cents => (CentsFract, Cents),
    CentsSigned => (CentsFract, CentsSigned),
    CoinDays => (CoinDays, CoinDays),
    CoinBlocks => (CoinBlocks, CoinBlocks),
    DollarBlocks => (DollarBlocks, DollarBlocks),
    DollarDaysPerCoin => (DollarDaysPerCoin, DollarDaysPerCoin),
    Float64 => (Float64, Float64),
}

/// A stored fixed-point ratio and its public representations.
pub trait FixedRatio: NumericValue + JsonSchema + Into<f32> {
    /// The stored series' name suffix: every fixed ratio is in parts per million.
    const SUFFIX: &'static str = "ppm";
}

impl FixedRatio for PartsPerMillion32 {}
impl FixedRatio for PriceRatio {}
impl FixedRatio for PartsPerMillionSigned32 {}
impl FixedRatio for PartsPerMillion64 {}
impl FixedRatio for PartsPerMillionSigned64 {}
