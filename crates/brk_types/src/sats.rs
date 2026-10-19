use std::ops::{Mul, SubAssign};

use bitcoin::Amount;
use derive_more::{Add, AddAssign, Deref, From, Into, Sub, Sum};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Bitcoin, Cents, CentsSats};
use crate::{CheckedSub, unlikely};

#[cfg(feature = "storage")]
use vecdb::{OverflowVecValue, Pco, SaturatingAdd, Version};

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
    Add,
    AddAssign,
    Sub,
    Sum,
    From,
    Into,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(
    example = &0,
    example = &546,
    example = &10000,
    example = &100_000_000,
    example = &2_100_000_000_000_000_u64
))]
pub struct Sats(u64);

#[cfg(feature = "storage")]
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
        debug_assert!(index < 1 << 31, "overflow sidecar index is too large");
        (1 << 31) | index as u32
    }
}

crate::impl_store_value!(Sats, u64);

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
    pub const FIFTY_BTC: Self = Self(50_00_000_000);
    pub const ONE_BTC_U64: u64 = 1_00_000_000;
    pub const ONE_BTC_U128: u128 = 1_00_000_000;
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

    pub fn is_max(&self) -> bool {
        *self == Self::MAX
    }

    /// The amount's value at `price` (cents per BTC), rounded to the nearest cent; NaN at a NaN
    /// price. Widens to u128 when the u64 product or its rounding offset overflows.
    #[inline(always)]
    pub fn value_at(self, price: Cents) -> Cents {
        if unlikely(price.is_nan()) {
            Cents::NAN
        } else if let Some(value) = self
            .0
            .checked_mul(price.inner())
            .and_then(|value| value.checked_add(Self::ONE_BTC_U64 / 2))
        {
            Cents::from(value / Self::ONE_BTC_U64)
        } else {
            CentsSats::from_price_sats(price, self).to_cents()
        }
    }
}

impl CheckedSub for Sats {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self::from)
    }
}

#[cfg(feature = "storage")]
impl SaturatingAdd for Sats {
    fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl SubAssign for Sats {
    fn sub_assign(&mut self, rhs: Self) {
        debug_assert!(self.0 >= rhs.0, "Sats underflow: {self} - {rhs}");
        self.0 -= rhs.0;
    }
}

impl Mul<f64> for Sats {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Sats::from((self.0 as f64 * rhs) as u64)
    }
}

impl From<u8> for Sats {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value as u64)
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

impl From<u128> for Sats {
    #[inline]
    fn from(value: u128) -> Self {
        debug_assert!(value <= u64::MAX as u128, "u128 value does not fit Sats");
        Self(value as u64)
    }
}

impl From<Sats> for u128 {
    #[inline]
    fn from(value: Sats) -> Self {
        value.0 as u128
    }
}

crate::impl_int_format!(Sats);
