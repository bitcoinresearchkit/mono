use std::ops::{Add, AddAssign, Div};

use bitview_primitives::{
    BasisPoints32, Bytes, Bytes32, BytesFract, CentsFract, CoinBlocks, CoinDays, Count, Count16,
    Count32, CountFract, Float64, PartsPerMillion32, PartsPerMillion64, PartsPerMillionSigned32,
    PartsPerMillionSigned64, Percent, Percent64, PriceRatio, Ratio, SatsFract, Seconds,
    SecondsFract, SigOps64, SigOpsFract, VSizeFract, Weight64, WeightFract,
};
use bitview_transforms::{FixedToPercent, FixedToRatio};
use brk_types::{Cents, CentsSigned, Sats, VSize, Weight};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{CheckedSub, Formattable, PcoVecValue, UnaryTransform};

pub trait ComputedVecValue
where
    Self: PcoVecValue
        + From<usize>
        + Div<usize, Output = Self>
        + Add<Output = Self>
        + AddAssign
        + Ord
        + Formattable
        + Serialize,
{
}
impl<T> ComputedVecValue for T where
    T: PcoVecValue
        + From<usize>
        + Div<usize, Output = Self>
        + Add<Output = Self>
        + AddAssign
        + Ord
        + Formattable
        + Serialize
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

impl Quantity for Count {
    type Fract = CountFract;
    type Sum = Count;
}

impl Quantity for Count16 {
    type Fract = CountFract;
    type Sum = Count;
}

impl Quantity for Count32 {
    type Fract = CountFract;
    type Sum = Count;
}

impl Quantity for Bytes {
    type Fract = BytesFract;
    type Sum = Bytes;
}

impl Quantity for Bytes32 {
    type Fract = BytesFract;
    type Sum = Bytes;
}

impl Quantity for VSize {
    type Fract = VSizeFract;
    type Sum = VSize;
}

impl Quantity for Weight64 {
    type Fract = WeightFract;
    type Sum = Weight64;
}

impl Quantity for Weight {
    type Fract = WeightFract;
    type Sum = Weight64;
}

impl Quantity for SigOps64 {
    type Fract = SigOpsFract;
    type Sum = SigOps64;
}

impl Quantity for Seconds {
    type Fract = SecondsFract;
    type Sum = Seconds;
}

impl Quantity for Sats {
    type Fract = SatsFract;
    type Sum = Sats;
}

impl Quantity for Cents {
    type Fract = CentsFract;
    type Sum = Cents;
}

impl Quantity for CentsSigned {
    type Fract = CentsFract;
    type Sum = CentsSigned;
}

impl Quantity for Percent64 {
    type Fract = Percent;
    type Sum = Percent64;
}

impl Quantity for CoinDays {
    type Fract = CoinDays;
    type Sum = CoinDays;
}

impl Quantity for CoinBlocks {
    type Fract = CoinBlocks;
    type Sum = CoinBlocks;
}

impl Quantity for Float64 {
    type Fract = Float64;
    type Sum = Float64;
}

/// A stored fixed-point ratio and its public representations.
pub trait FixedRatio: NumericValue + JsonSchema {
    const SUFFIX: &'static str;

    type ToRatio: UnaryTransform<Self, Ratio>;
    type ToPercent: UnaryTransform<Self, Percent>;
}

impl FixedRatio for PartsPerMillion32 {
    const SUFFIX: &'static str = "ppm";

    type ToRatio = FixedToRatio;
    type ToPercent = FixedToPercent;
}

impl FixedRatio for PriceRatio {
    const SUFFIX: &'static str = "ppm";

    type ToRatio = FixedToRatio;
    type ToPercent = FixedToPercent;
}

impl FixedRatio for BasisPoints32 {
    const SUFFIX: &'static str = "bps";

    type ToRatio = FixedToRatio;
    type ToPercent = FixedToPercent;
}

impl FixedRatio for PartsPerMillionSigned32 {
    const SUFFIX: &'static str = "ppm";

    type ToRatio = FixedToRatio;
    type ToPercent = FixedToPercent;
}

impl FixedRatio for PartsPerMillion64 {
    const SUFFIX: &'static str = "ppm";

    type ToRatio = FixedToRatio;
    type ToPercent = FixedToPercent;
}

impl FixedRatio for PartsPerMillionSigned64 {
    const SUFFIX: &'static str = "ppm";

    type ToRatio = FixedToRatio;
    type ToPercent = FixedToPercent;
}
