use std::ops::{Add, AddAssign, Div, Sub, SubAssign};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{Bytes, Formattable};

use super::{Cents, CentsSquaredSats, Sats};

/// Cents × Sats (u128) - price in cents multiplied by amount in sats.
/// Uses u128 because large amounts at any price can overflow u64.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct CentsSats(u128);

impl CentsSats {
    pub const ZERO: Self = Self(0);

    #[inline(always)]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    /// Compute from price and sats using widening multiplication
    #[inline(always)]
    pub fn from_price_sats(price: Cents, sats: Sats) -> Self {
        Self(price.inner() as u128 * sats.as_u128())
    }

    #[inline(always)]
    pub const fn inner(self) -> u128 {
        self.0
    }

    #[inline(always)]
    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Convert to CentsUnsigned by dividing by ONE_BTC.
    #[inline(always)]
    pub fn to_cents(self) -> Cents {
        Cents::new((self.0 / Sats::ONE_BTC_U128) as u64)
    }

    /// Convert to cents, rounding to the nearest cent.
    #[inline(always)]
    pub fn to_cents_rounded(self) -> Cents {
        Cents::new(((self.0 + Sats::ONE_BTC_U128 / 2) / Sats::ONE_BTC_U128) as u64)
    }

    /// Get the realized price (cents per BTC) given the sats amount.
    #[inline(always)]
    pub fn realized_price(self, sats: Sats) -> Cents {
        if sats.is_zero() {
            return Cents::ZERO;
        }
        let result = self.0 / sats.as_u128();
        Cents::new(result.min(u32::MAX as u128) as u64)
    }

    /// Compute capitalized cap (price² × sats) = price × (price × sats)
    #[inline(always)]
    pub fn to_capitalized_cap(self, price: Cents) -> CentsSquaredSats {
        CentsSquaredSats::new(price.inner() as u128 * self.0)
    }
}

impl Add for CentsSats {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for CentsSats {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for CentsSats {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for CentsSats {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl From<u128> for CentsSats {
    #[inline(always)]
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<CentsSats> for u128 {
    #[inline(always)]
    fn from(value: CentsSats) -> Self {
        value.0
    }
}

impl Div<usize> for CentsSats {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: usize) -> Self {
        Self(self.0 / rhs as u128)
    }
}

impl Formattable for CentsSats {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = itoa::Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}

impl Bytes for CentsSats {
    type Array = [u8; 16];

    fn to_bytes(&self) -> Self::Array {
        self.0.to_le_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        Ok(Self(u128::from_bytes(bytes)?))
    }
}

impl std::fmt::Display for CentsSats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_to_nearest_cent() {
        assert_eq!(
            CentsSats::new(Sats::ONE_BTC_U128 / 2 - 1).to_cents_rounded(),
            Cents::ZERO
        );
        assert_eq!(
            CentsSats::new(Sats::ONE_BTC_U128 / 2).to_cents_rounded(),
            Cents::new(1)
        );
    }
}
