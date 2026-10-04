use std::ops::{Div, Mul};

use brk_types::{CheckedSub, Dollars};

use crate::{Close, scalar::scalar};

scalar! {
    /// Fractional satoshis (f64): mean amounts in sats, and USD prices expressed in sats.
    ///
    /// A USD price in sats is `usd_value * 100_000_000 / btc_price`
    ///
    /// When BTC is $100,000:
    /// - $1 = 1,000 sats
    /// - $0.001 = 1 sat
    /// - $0.0001 = 0.1 sats (fractional)
    float SatsFract(f64)
}

impl SatsFract {
    pub const ONE_BTC: Self = Self(100_000_000.0);
}

impl From<f32> for SatsFract {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}

impl From<SatsFract> for f32 {
    #[inline]
    fn from(value: SatsFract) -> Self {
        value.0 as f32
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

impl CheckedSub<usize> for SatsFract {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        Some(Self(self.0 - rhs as f64))
    }
}

#[cfg(feature = "storage")]
impl vecdb::CheckedSub<usize> for SatsFract {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        CheckedSub::checked_sub(self, rhs)
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
