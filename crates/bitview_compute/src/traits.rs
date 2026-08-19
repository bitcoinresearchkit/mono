use std::ops::{Add, AddAssign, Div};

use brk_types::{
    PartsPerMillion32, PartsPerMillion64, PartsPerMillionSigned32, PartsPerMillionSigned64,
    StoredF32,
};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{CheckedSub, Formattable, PcoVecValue, UnaryTransform};

use crate::{FixedToPercent, FixedToRatio};

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

/// A stored fixed-point ratio and its public representations.
pub trait FixedRatio: NumericValue + JsonSchema {
    const SUFFIX: &'static str;

    type ToRatio: UnaryTransform<Self, StoredF32>;
    type ToPercent: UnaryTransform<Self, StoredF32>;
}

impl FixedRatio for PartsPerMillion32 {
    const SUFFIX: &'static str = "ppm";

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
