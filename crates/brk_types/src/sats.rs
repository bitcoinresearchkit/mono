use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use bitcoin::Amount;
use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{CheckedSub, Formattable, OverflowVecValue, Pco, SaturatingAdd, Version};

use crate::{StoredF64, StoredU64};

use super::{Bitcoin, Cents, Dollars, Height};

/// Amount in satoshis (1 BTC = 100,000,000 sats)
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    Default,
    Serialize,
    Deserialize,
    Hash,
    Pco,
    JsonSchema,
)]
#[schemars(
    example = &0,
    example = &546,
    example = &10000,
    example = &100_000_000,
    example = &2_100_000_000_000_000_u64
)]
pub struct Sats(u64);

impl OverflowVecValue for Sats {
    type Compact = u32;

    const VERSION: Version = Version::ONE;

    #[inline(always)]
    fn to_compact(&self) -> Option<Self::Compact> {
        (self.0 < 1_u64 << 31).then_some(self.0 as u32)
    }

    #[inline(always)]
    fn from_compact(compact: Self::Compact) -> Self {
        Self(u64::from(compact))
    }

    #[inline(always)]
    fn overflow_index(compact: Self::Compact) -> Option<usize> {
        (compact & (1 << 31) != 0).then_some((compact & !(1 << 31)) as usize)
    }

    #[inline(always)]
    fn from_overflow_index(index: usize) -> Self::Compact {
        assert!(index < 1 << 31, "overflow sidecar index is too large");
        (1 << 31) | index as u32
    }
}

#[allow(clippy::inconsistent_digit_grouping)]
impl Sats {
    pub const ZERO: Self = Self(0);
    pub const _1: Self = Self(1);
    pub const _10: Self = Self(10);
    pub const _100: Self = Self(100);
    pub const _1K: Self = Self(1_000);
    pub const _10K: Self = Self(10_000);
    pub const _100K: Self = Self(100_000);
    pub const _1M: Self = Self(1_000_000);
    pub const _10M: Self = Self(10_000_000);
    pub const _1BTC: Self = Self::ONE_BTC;
    pub const _10BTC: Self = Self(10_00_000_000);
    pub const _100BTC: Self = Self(100_00_000_000);
    pub const _1K_BTC: Self = Self(1_000_00_000_000);
    pub const _10K_BTC: Self = Self(10_000_00_000_000);
    pub const _100K_BTC: Self = Self(100_000_00_000_000);
    pub const ONE_BTC: Self = Self(1_00_000_000);
    pub const MAX: Self = Self(u64::MAX);
    pub const COINBASE: Self = Self(u64::MAX);
    pub const FIFTY_BTC: Self = Self(50_00_000_000);
    pub const ONE_BTC_U64: u64 = 1_00_000_000;
    pub const ONE_BTC_U128: u128 = 1_00_000_000;
    pub const ONE_BTC_I128: i128 = 1_00_000_000;

    pub fn new(sats: u64) -> Self {
        Self(sats)
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    pub fn is_not_zero(&self) -> bool {
        *self != Self::ZERO
    }

    #[inline(always)]
    pub const fn as_u128(self) -> u128 {
        self.0 as u128
    }

    #[inline(always)]
    pub const fn as_i128(self) -> i128 {
        self.0 as i128
    }

    pub fn is_max(&self) -> bool {
        *self == Self::MAX
    }
}

impl Add for Sats {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Sats {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign for Sats {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl CheckedSub for Sats {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self::from)
    }
}

impl CheckedSub<usize> for Sats {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        self.0.checked_sub(rhs as u64).map(Self::from)
    }
}

impl SaturatingAdd for Sats {
    fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl SubAssign for Sats {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.checked_sub(rhs).unwrap_or_else(|| {
            panic!("Sats underflow: {} - {} would be negative", self, rhs);
        });
    }
}

impl Mul<Sats> for Sats {
    type Output = Self;
    fn mul(self, rhs: Sats) -> Self::Output {
        Sats::from(self.0.checked_mul(rhs.0).unwrap())
    }
}

impl Mul<usize> for Sats {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Sats::from(self.0.checked_mul(rhs as u64).unwrap())
    }
}

impl Mul<u8> for Sats {
    type Output = Self;
    fn mul(self, rhs: u8) -> Self::Output {
        Sats::from(self.0.checked_mul(rhs as u64).unwrap())
    }
}

impl Mul<u64> for Sats {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self::Output {
        Sats::from(self.0.checked_mul(rhs).unwrap())
    }
}

impl Mul<Height> for Sats {
    type Output = Self;
    fn mul(self, rhs: Height) -> Self::Output {
        Sats::from(self.0.checked_mul(u64::from(rhs)).unwrap())
    }
}

impl Mul<f64> for Sats {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Sats::from((self.0 as f64 * rhs) as u64)
    }
}

impl Mul<StoredF64> for Sats {
    type Output = Self;
    fn mul(self, rhs: StoredF64) -> Self::Output {
        self * f64::from(rhs)
    }
}

impl Sum for Sats {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sats: u64 = iter.map(|sats| sats.0).sum();
        Self::from(sats)
    }
}

impl Div<Dollars> for Sats {
    type Output = Self;
    #[allow(
        clippy::suspicious_arithmetic_impl,
        reason = "cents-precision upscale before division"
    )]
    fn div(self, rhs: Dollars) -> Self::Output {
        let raw_cents = u64::from(Cents::from(rhs));
        (self.0 * 100)
            .checked_div(raw_cents)
            .map(Self)
            .unwrap_or(Self::MAX)
    }
}

impl Div<Sats> for Sats {
    type Output = Self;
    fn div(self, rhs: Sats) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap_or(0))
    }
}

impl Div<usize> for Sats {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0.checked_div(rhs as u64).unwrap_or(0))
    }
}

impl Div<StoredU64> for Sats {
    type Output = Self;
    fn div(self, rhs: StoredU64) -> Self::Output {
        Self(self.0.checked_div(u64::from(rhs)).unwrap_or(0))
    }
}

impl From<u8> for Sats {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value as u64)
    }
}

impl From<u64> for Sats {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<usize> for Sats {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<f32> for Sats {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value.round() as u64)
    }
}

impl From<f64> for Sats {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value.round() as u64)
    }
}

impl From<Sats> for f64 {
    #[inline]
    fn from(value: Sats) -> Self {
        value.0 as f64
    }
}

impl From<Sats> for usize {
    #[inline]
    fn from(value: Sats) -> Self {
        value.0 as usize
    }
}

impl From<Amount> for Sats {
    #[inline]
    fn from(value: Amount) -> Self {
        Self(value.to_sat())
    }
}
impl From<Sats> for Amount {
    #[inline]
    fn from(value: Sats) -> Self {
        Self::from_sat(value.0)
    }
}

impl From<Bitcoin> for Sats {
    #[inline]
    fn from(value: Bitcoin) -> Self {
        Self((f64::from(value) * (Sats::ONE_BTC.0 as f64)).round() as u64)
    }
}

impl From<Sats> for u64 {
    #[inline]
    fn from(value: Sats) -> Self {
        value.0
    }
}

impl From<u128> for Sats {
    #[inline]
    fn from(value: u128) -> Self {
        if value > u64::MAX as u128 {
            panic!("u128 bigger than u64")
        }
        Self(value as u64)
    }
}

impl From<Sats> for u128 {
    #[inline]
    fn from(value: Sats) -> Self {
        value.0 as u128
    }
}

impl Mul<Sats> for usize {
    type Output = Sats;
    fn mul(self, rhs: Sats) -> Self::Output {
        Self::Output::from(rhs.0 * self as u64)
    }
}

impl std::fmt::Display for Sats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = itoa::Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

impl Formattable for Sats {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = itoa::Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_encoding_preserves_inline_values_and_sidecar_indexes() {
        for value in [0, (1_u64 << 31) - 1] {
            let value = Sats::from(value);
            let compact = value.to_compact().unwrap();
            assert_eq!(Sats::overflow_index(compact), None);
            assert_eq!(Sats::from_compact(compact), value);
        }

        assert_eq!(Sats::from(1_u64 << 31).to_compact(), None);
        assert_eq!(Sats::MAX.to_compact(), None);

        for index in [0, (1_usize << 31) - 1] {
            let compact = Sats::from_overflow_index(index);
            assert_eq!(Sats::overflow_index(compact), Some(index));
        }
    }
}
