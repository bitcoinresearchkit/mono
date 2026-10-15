use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Cents, Sats};

#[cfg(feature = "storage")]
use vecdb::Result;

#[cfg(feature = "storage")]
use vecdb::Bytes;

/// Cents × Sats (u128) - price in cents multiplied by amount in sats.
/// Uses u128 because large amounts at any price can overflow u64.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    From,
    Into,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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

    /// The realized price (cents per BTC) given the sats amount; undefined (NaN) without sats.
    #[inline(always)]
    pub fn realized_price(self, sats: Sats) -> Cents {
        if sats.is_zero() {
            return Cents::NAN;
        }
        Cents::from(self.0 / sats.as_u128())
    }
}

crate::impl_int_format!(CentsSats);

#[cfg(feature = "storage")]
// SAFETY: not native layout.
unsafe impl Bytes for CentsSats {
    type Array = [u8; 16];

    fn to_bytes(&self) -> Self::Array {
        self.0.to_le_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(u128::from_bytes(bytes)?))
    }
}
