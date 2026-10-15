use derive_more::{Add, AddAssign, From, Into};
use std::ops::Mul;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Sats;
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::Pco;

/// Bitcoin amount as floating point (1 BTC = 100,000,000 satoshis)
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Add, AddAssign, From, Into)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Bitcoin(f64);

impl Mul<usize> for Bitcoin {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as f64)
    }
}

impl From<Sats> for Bitcoin {
    #[inline]
    fn from(value: Sats) -> Self {
        Self(f64::from(value) / (f64::from(Sats::ONE_BTC)))
    }
}

impl From<usize> for Bitcoin {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}

crate::impl_float_ord_format!(Bitcoin);

impl CheckedSub<usize> for Bitcoin {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        Some(Self(self.0 - rhs as f64))
    }
}
impl CheckedSub<Bitcoin> for Bitcoin {
    fn checked_sub(self, rhs: Bitcoin) -> Option<Self> {
        Some(Self(self.0 - rhs.0))
    }
}
