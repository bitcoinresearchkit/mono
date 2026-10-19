use derive_more::{Add, AddAssign, From, Into, Sub, SubAssign};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Cents, Sats};

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
#[cfg_attr(feature = "storage", derive(Bytes))]
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

    /// The amount in cents (divided by one BTC), rounded to the nearest cent.
    #[inline(always)]
    pub fn to_cents(self) -> Cents {
        Cents::from(div_round(self.0, Sats::ONE_BTC_U128))
    }

    /// The realized price (cents per BTC) given the sats amount, rounded to the nearest cent;
    /// undefined (NaN) without sats.
    #[inline(always)]
    pub fn realized_price(self, sats: Sats) -> Cents {
        if sats.is_zero() {
            return Cents::NAN;
        }
        Cents::from(div_round(self.0, sats.as_u128()))
    }
}

/// `numerator / denominator` rounded to the nearest integer (halves up).
#[inline(always)]
fn div_round(numerator: u128, denominator: u128) -> u128 {
    (numerator + denominator / 2) / denominator
}

crate::impl_int_format!(CentsSats);
