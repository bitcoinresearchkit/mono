use derive_more::{Add, AddAssign, From, Into};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Sats;
use crate::CheckedSub;

#[cfg(feature = "storage")]
use vecdb::Pco;

/// An amount in bitcoin, as a decimal number (1 BTC = 100,000,000 satoshis).
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Add, AddAssign, From, Into)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct Bitcoin(f64);

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

impl CheckedSub<Bitcoin> for Bitcoin {
    fn checked_sub(self, rhs: Bitcoin) -> Option<Self> {
        Some(Self(self.0 - rhs.0))
    }
}
