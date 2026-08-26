use std::{
    cmp::Ordering,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul},
};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use vecdb::{CheckedSub, Formattable, Pco};

use super::{Sats, VSize, Weight};

const MILLIS_PER_SAT_VBYTE: u64 = 1_000;

/// Fee rate stored in milli-sat/vB and exposed as sat/vB.
#[derive(Debug, Default, Clone, Copy, Pco, JsonSchema)]
#[repr(transparent)]
#[schemars(
    with = "f64",
    example = &0.1,
    example = &1.0,
    example = &2.5,
    example = &10.14,
    example = &25.0,
    example = &302.11
)]
pub struct FeeRate(u64);

impl FeeRate {
    pub const ZERO: Self = Self(0);
    pub const MIN: Self = Self(100);
    pub const NAN: Self = Self(u64::MAX);
    pub const MAX_FINITE: Self = Self(u64::MAX - 1);

    #[inline]
    pub fn new(sats_per_vbyte: f64) -> Self {
        Self::from(sats_per_vbyte)
    }

    #[inline]
    pub const fn from_milli(milli_sats_per_vbyte: u64) -> Self {
        assert!(
            milli_sats_per_vbyte != u64::MAX,
            "u64::MAX is reserved as FeeRate::NAN"
        );
        Self(milli_sats_per_vbyte)
    }

    #[inline]
    pub const fn milli(self) -> Option<u64> {
        if self.is_nan() { None } else { Some(self.0) }
    }

    #[inline]
    pub const fn is_nan(self) -> bool {
        self.0 == u64::MAX
    }

    /// Round up to the nearest multiple of `nearest`.
    #[inline]
    pub fn ceil_to(self, nearest: Self) -> Self {
        if self.is_nan() || nearest.is_nan() || nearest.0 == 0 {
            return Self::NAN;
        }
        let remainder = self.0 % nearest.0;
        if remainder == 0 {
            self
        } else {
            Self::from_checked(self.0.checked_add(nearest.0 - remainder))
        }
    }

    /// Round to the nearest multiple of `nearest`.
    #[inline]
    pub fn round_to(self, nearest: Self) -> Self {
        if self.is_nan() || nearest.is_nan() || nearest.0 == 0 {
            return Self::NAN;
        }
        let remainder = self.0 % nearest.0;
        if remainder < nearest.0.div_ceil(2) {
            Self(self.0 - remainder)
        } else {
            Self::from_checked(self.0.checked_add(nearest.0 - remainder))
        }
    }

    /// Values are canonical milli-sat/vB already.
    #[inline]
    pub const fn round_milli(self) -> Self {
        self
    }

    /// Arithmetic mean rounded to the nearest milli-sat/vB.
    #[inline]
    pub fn mean(a: Self, b: Self) -> Self {
        if a.is_nan() || b.is_nan() {
            Self::NAN
        } else {
            let sum = u128::from(a.0) + u128::from(b.0);
            Self(sum.div_ceil(2) as u64)
        }
    }

    #[inline]
    fn from_checked(value: Option<u64>) -> Self {
        match value {
            Some(value) if value != u64::MAX => Self(value),
            _ => Self::NAN,
        }
    }

    #[inline]
    fn divide_round(self, divisor: u128) -> Self {
        if self.is_nan() || divisor == 0 {
            return Self::NAN;
        }
        let value = (u128::from(self.0) + divisor / 2) / divisor;
        Self(value as u64)
    }
}

impl From<(Sats, VSize)> for FeeRate {
    #[inline]
    fn from((sats, vsize): (Sats, VSize)) -> Self {
        if sats.is_zero() {
            return Self::ZERO;
        }
        let vsize = u64::from(vsize);
        if vsize == 0 {
            return Self::NAN;
        }
        if let Some(numerator) = u64::from(sats).checked_mul(MILLIS_PER_SAT_VBYTE) {
            return Self(numerator.div_ceil(vsize));
        }
        let milli = (sats.as_u128() * u128::from(MILLIS_PER_SAT_VBYTE)).div_ceil(u128::from(vsize));
        if milli >= u128::from(u64::MAX) {
            Self::NAN
        } else {
            Self(milli as u64)
        }
    }
}

impl From<(Sats, Weight)> for FeeRate {
    #[inline]
    fn from((sats, weight): (Sats, Weight)) -> Self {
        Self::from((sats, VSize::from(weight.to_vbytes_ceil())))
    }
}

impl From<f64> for FeeRate {
    #[inline]
    fn from(value: f64) -> Self {
        if !value.is_finite() || value < 0.0 {
            return Self::NAN;
        }
        let milli = (value * MILLIS_PER_SAT_VBYTE as f64).round();
        if milli >= u64::MAX as f64 {
            Self::MAX_FINITE
        } else {
            Self(milli as u64)
        }
    }
}

impl From<FeeRate> for f64 {
    #[inline]
    fn from(value: FeeRate) -> Self {
        if value.is_nan() {
            f64::NAN
        } else {
            value.0 as f64 / MILLIS_PER_SAT_VBYTE as f64
        }
    }
}

impl From<usize> for FeeRate {
    #[inline]
    fn from(value: usize) -> Self {
        Self::from(value as f64)
    }
}

impl Add for FeeRate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_nan() || rhs.is_nan() {
            Self::NAN
        } else {
            Self::from_checked(self.0.checked_add(rhs.0))
        }
    }
}

impl AddAssign for FeeRate {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sum for FeeRate {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

impl Div<usize> for FeeRate {
    type Output = Self;

    fn div(self, rhs: usize) -> Self::Output {
        self.divide_round(rhs as u128)
    }
}

impl Div<f64> for FeeRate {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::from(f64::from(self) / rhs)
    }
}

impl Mul<f64> for FeeRate {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::from(f64::from(self) * rhs)
    }
}

impl PartialEq for FeeRate {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FeeRate {}

impl PartialOrd for FeeRate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FeeRate {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.is_nan(), other.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.0.cmp(&other.0),
        }
    }
}

impl CheckedSub for FeeRate {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        if self.is_nan() || rhs.is_nan() {
            None
        } else {
            self.0.checked_sub(rhs.0).map(Self)
        }
    }
}

impl Serialize for FeeRate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(f64::from(*self))
    }
}

impl<'de> Deserialize<'de> for FeeRate {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(f64::deserialize(deserializer)?))
    }
}

impl std::fmt::Display for FeeRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = ryu::Buffer::new();
        f.write_str(buf.format(f64::from(*self)))
    }
}

impl Formattable for FeeRate {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        if !self.is_nan() {
            let mut value = ryu::Buffer::new();
            buf.extend_from_slice(value.format(f64::from(*self)).as_bytes());
        }
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if self.is_nan() {
            buf.extend_from_slice(b"null");
        } else {
            self.write_to(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_rate_is_canonical_milli_sat_per_vbyte() {
        assert_eq!(size_of::<FeeRate>(), size_of::<u64>());
        assert_eq!(FeeRate::new(10.1404), FeeRate::from_milli(10_140));
        assert_eq!(FeeRate::new(10.1406), FeeRate::from_milli(10_141));
        assert_eq!(
            FeeRate::from((Sats::from(1_u64), VSize::from(3_u64))),
            FeeRate::from_milli(334)
        );
    }

    #[test]
    fn fee_rate_keeps_decimal_json() {
        let value = FeeRate::new(10.14);
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, "10.14");
        assert_eq!(serde_json::from_str::<FeeRate>(&json).unwrap(), value);
        assert_eq!(serde_json::to_string(&FeeRate::NAN).unwrap(), "null");
    }

    #[test]
    fn fee_rate_integer_rounding_matches_milli_precision() {
        assert_eq!(
            FeeRate::mean(FeeRate::from_milli(1), FeeRate::from_milli(2)),
            FeeRate::from_milli(2)
        );
        assert_eq!(
            FeeRate::from_milli(1_234).ceil_to(FeeRate::from_milli(10)),
            FeeRate::from_milli(1_240)
        );
        assert_eq!(
            FeeRate::from_milli(1_234).round_to(FeeRate::from_milli(10)),
            FeeRate::from_milli(1_230)
        );
        assert!(FeeRate::NAN < FeeRate::ZERO);
    }

    #[test]
    fn fee_rate_fast_path_matches_u128_arithmetic() {
        for index in 0..1_200_000_u64 {
            let sats = index.wrapping_mul(6_364_136_223_846_793_005);
            let vsize = index.wrapping_mul(2_654_435_761) % 4_000_001;
            let expected = if sats == 0 {
                FeeRate::ZERO
            } else if vsize == 0 {
                FeeRate::NAN
            } else {
                let milli = (u128::from(sats) * u128::from(MILLIS_PER_SAT_VBYTE))
                    .div_ceil(u128::from(vsize));
                if milli >= u128::from(u64::MAX) {
                    FeeRate::NAN
                } else {
                    FeeRate::from_milli(milli as u64)
                }
            };
            assert_eq!(
                FeeRate::from((Sats::from(sats), VSize::from(vsize))),
                expected,
            );
        }
    }
}
