use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use brk_types::{Cents, CentsSats};
#[cfg(feature = "storage")]
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::Result;

#[cfg(feature = "storage")]
use vecdb::{Bytes, Formattable};

/// Raw cents squared (u128) - stores cents² × sats without division.
/// Used for precise accumulation of capitalized cap values: Σ(price² × sats).
/// capitalized_price = capitalized_cap_raw / realized_cap_raw
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct CentsSquaredSats(u128);

impl CentsSquaredSats {
    #[inline(always)]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub const fn inner(self) -> u128 {
        self.0
    }

    /// Capitalized cap (price² × sats) = price × (price × sats)
    #[inline(always)]
    pub fn from_price_cents_sats(price: Cents, value: CentsSats) -> Self {
        Self(price.inner() as u128 * value.inner())
    }
}

impl Div<u128> for CentsSquaredSats {
    type Output = u128;
    #[inline(always)]
    fn div(self, rhs: u128) -> u128 {
        self.0 / rhs
    }
}

impl AddAssign<u128> for CentsSquaredSats {
    #[inline(always)]
    fn add_assign(&mut self, rhs: u128) {
        self.0 += rhs;
    }
}

impl SubAssign<u128> for CentsSquaredSats {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: u128) {
        self.0 -= rhs;
    }
}

impl Add for CentsSquaredSats {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for CentsSquaredSats {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for CentsSquaredSats {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for CentsSquaredSats {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl From<u128> for CentsSquaredSats {
    #[inline(always)]
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<CentsSquaredSats> for u128 {
    #[inline(always)]
    fn from(value: CentsSquaredSats) -> Self {
        value.0
    }
}

impl From<usize> for CentsSquaredSats {
    #[inline(always)]
    fn from(value: usize) -> Self {
        Self(value as u128)
    }
}

impl Div<usize> for CentsSquaredSats {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: usize) -> Self {
        Self(self.0 / rhs as u128)
    }
}

#[cfg(feature = "storage")]
impl Formattable for CentsSquaredSats {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

#[cfg(feature = "storage")]
impl Bytes for CentsSquaredSats {
    type Array = [u8; 16];

    fn to_bytes(&self) -> Self::Array {
        self.0.to_le_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(u128::from_bytes(bytes)?))
    }
}

impl Display for CentsSquaredSats {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
