use std::{
    cmp::Ordering,
    f64,
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use brk_types::{CheckedSub, Dollars};
use derive_more::Deref;
use ryu::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Close;

#[cfg(feature = "storage")]
use vecdb::CheckedSub as VecdbCheckedSub;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Fractional satoshis (f64) - for representing USD prices in sats
///
/// Formula: `sats_fract = usd_value * 100_000_000 / btc_price`
///
/// When BTC is $100,000:
/// - $1 = 1,000 sats
/// - $0.001 = 1 sat
/// - $0.0001 = 0.1 sats (fractional)
#[derive(Debug, Deref, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct SatsFract(f64);

impl SatsFract {
    const NAN: Self = Self(f64::NAN);
    pub const ONE_BTC: Self = Self(100_000_000.0);
}

impl From<f64> for SatsFract {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<f32> for SatsFract {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}

impl From<usize> for SatsFract {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}

impl From<SatsFract> for f64 {
    #[inline]
    fn from(value: SatsFract) -> Self {
        value.0
    }
}

impl From<SatsFract> for f32 {
    #[inline]
    fn from(value: SatsFract) -> Self {
        value.0 as f32
    }
}

impl Add for SatsFract {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for SatsFract {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for SatsFract {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul for SatsFract {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<usize> for SatsFract {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as f64)
    }
}

impl Div<usize> for SatsFract {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        if rhs == 0 {
            Self::NAN
        } else {
            Self(self.0 / rhs as f64)
        }
    }
}

impl Div for SatsFract {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0.0 {
            Self::NAN
        } else {
            Self(self.0 / rhs.0)
        }
    }
}

impl CheckedSub for SatsFract {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0 - rhs.0))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub for SatsFract {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl CheckedSub<usize> for SatsFract {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        Some(Self(self.0 - rhs as f64))
    }
}
#[cfg(feature = "storage")]
impl VecdbCheckedSub<usize> for SatsFract {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
    }
}

impl PartialEq for SatsFract {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => true,
            (true, false) | (false, true) => false,
            (false, false) => self.0 == other.0,
        }
    }
}

impl Eq for SatsFract {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl PartialOrd for SatsFract {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for SatsFract {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

impl Sum for SatsFract {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|v| v.0).sum::<f64>())
    }
}

impl Display for SatsFract {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

#[cfg(feature = "storage")]
impl Formattable for SatsFract {
    const UNDEFINED: bool = true;

    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        if self.0.is_finite() {
            let mut b = Buffer::new();
            buf.extend_from_slice(b.format(self.0).as_bytes());
        }
    }

    #[inline(always)]
    fn fmt_json(&self, buf: &mut Vec<u8>) {
        if self.0.is_finite() {
            self.write_to(buf);
        } else {
            buf.extend_from_slice(b"null");
        }
    }
}

impl Div<Dollars> for SatsFract {
    type Output = Self;
    fn div(self, rhs: Dollars) -> Self::Output {
        let rhs = f64::from(rhs);
        if rhs == 0.0 {
            Self::NAN
        } else {
            Self(self.0 / rhs)
        }
    }
}

impl Div<Close<Dollars>> for SatsFract {
    type Output = Self;
    fn div(self, rhs: Close<Dollars>) -> Self::Output {
        self / *rhs
    }
}
