use std::{
    fmt::{Display, Formatter, Result},
    ops::Sub,
};

use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Cents, Dollars};
use crate::unlikely;

/// Compact unsigned cents (u32) - memory-efficient for map keys.
/// Supports finite values from $0.00 to $42,949,672.94.
/// `u32::MAX` is reserved as a NaN sentinel.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct CentsCompact(u32);

impl CentsCompact {
    pub const ZERO: Self = Self(0);
    pub const NAN: Self = Self(u32::MAX);
    #[inline]
    pub const fn new(value: u32) -> Self {
        assert!(
            value != u32::MAX,
            "u32::MAX is reserved as CentsCompact::NAN"
        );
        Self(value)
    }

    #[inline]
    pub const fn is_nan(self) -> bool {
        self.0 == u32::MAX
    }

    #[inline]
    pub const fn finite_inner(self) -> Option<u32> {
        if self.is_nan() { None } else { Some(self.0) }
    }

    #[inline]
    pub const fn inner(self) -> u32 {
        match self.finite_inner() {
            Some(value) => value,
            None => panic!("CentsCompact::NAN has no finite integer representation"),
        }
    }

    #[inline(always)]
    pub const fn as_u128(self) -> u128 {
        self.inner() as u128
    }

    #[inline]
    pub fn to_dollars(self) -> Dollars {
        if unlikely(self.is_nan()) {
            Dollars::NAN
        } else {
            Dollars::from(self.0 as f64 / 100.0)
        }
    }

    /// Round to nearest dollar, then apply N significant digits.
    /// E.g., 12345 (= $123.45) → 12300 (= $123.00) with 5 digits
    /// E.g., 1234567 (= $12345.67) → 1234600 (= $12346.00) with 5 digits
    #[inline]
    pub fn round_to_dollar(self, digits: i32) -> Self {
        if unlikely(self.is_nan()) {
            return Self::NAN;
        }

        // Round to nearest dollar (nearest 100 cents)
        let dollars = (self.0 as u64 + 50) / 100;
        // Apply significant digit rounding to dollars, then convert back to cents
        let ilog10 = dollars.checked_ilog10().unwrap_or(0) as i32;
        let rounded_dollars = if ilog10 >= digits {
            let log_diff = ilog10 - digits + 1;
            let pow = 10u64.pow(log_diff as u32);
            ((dollars + pow / 2) / pow) * pow
        } else {
            dollars
        };
        Self::from_finite_u64(rounded_dollars * 100)
    }

    #[inline]
    fn from_finite_u64(value: u64) -> Self {
        assert!(
            value < u32::MAX as u64,
            "CentsCompact finite value exceeds compact range"
        );
        Self(value as u32)
    }
}

impl From<Dollars> for CentsCompact {
    #[inline]
    fn from(value: Dollars) -> Self {
        let f = f64::from(value);
        if unlikely(!f.is_finite()) {
            Self::NAN
        } else if f < 0.0 {
            Self::ZERO
        } else {
            let cents = (f * 100.0).round();
            assert!(
                cents < u32::MAX as f64,
                "price ${f} exceeds CentsCompact finite range"
            );
            Self(cents as u32)
        }
    }
}

impl From<CentsCompact> for Dollars {
    #[inline]
    fn from(value: CentsCompact) -> Self {
        value.to_dollars()
    }
}

impl From<u32> for CentsCompact {
    #[inline]
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<CentsCompact> for u32 {
    #[inline]
    fn from(value: CentsCompact) -> Self {
        value.inner()
    }
}

impl From<CentsCompact> for f64 {
    #[inline]
    fn from(value: CentsCompact) -> Self {
        if unlikely(value.is_nan()) {
            f64::NAN
        } else {
            value.0 as f64
        }
    }
}

impl From<Cents> for CentsCompact {
    #[inline]
    fn from(value: Cents) -> Self {
        if unlikely(value.is_nan()) {
            Self::NAN
        } else {
            Self::from_finite_u64(value.inner())
        }
    }
}

impl From<CentsCompact> for Cents {
    #[inline]
    fn from(value: CentsCompact) -> Self {
        if unlikely(value.is_nan()) {
            Cents::NAN
        } else {
            Cents::new(value.inner() as u64)
        }
    }
}

impl Sub for CentsCompact {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        if unlikely(self.is_nan() || rhs.is_nan()) {
            Self::NAN
        } else {
            Self(self.0 - rhs.0)
        }
    }
}

impl Display for CentsCompact {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_round_trips_between_cent_types() {
        assert_eq!(CentsCompact::from(Cents::NAN), CentsCompact::NAN);
        assert_eq!(Cents::from(CentsCompact::NAN), Cents::NAN);
        assert!(f64::from(CentsCompact::NAN).is_nan());
        assert!(f64::from(Dollars::from(CentsCompact::NAN)).is_nan());
    }

    #[test]
    fn nan_propagates_through_arithmetic() {
        let finite = CentsCompact::new(100);
        assert_eq!(CentsCompact::NAN - finite, CentsCompact::NAN);
        assert_eq!(CentsCompact::NAN.round_to_dollar(3), CentsCompact::NAN);
    }
}
