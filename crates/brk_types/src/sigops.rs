use std::{
    fmt::{Display, Formatter, Result},
    iter::Sum,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use derive_more::Deref;
use itoa::Buffer;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, SigOps32, VSize};

#[cfg(feature = "storage")]
use vecdb::{CheckedSub as VecdbCheckedSub, Formattable, Pco};

/// BIP-141 signature-operation cost: legacy and P2SH-redeem sigops count four and witness sigops
/// one. Sums over many transactions or blocks use the full 64 bits.
#[derive(
    Debug, Default, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct SigOps(u64);

impl SigOps {
    pub const ZERO: Self = Self(0);

    /// Vbytes per sigop under BIP-141 policy. Core's `nSigOpCost`
    /// adjustment factor: `adjusted_vsize = max(vsize, sigops * 5)`.
    const VBYTES_PER_SIGOP: u64 = 5;

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Policy-adjusted vsize: `max(vsize, sigops * 5)`. The denominator
    /// Core uses when ranking sigop-heavy txs at fixed fee.
    #[inline]
    pub fn adjust_vsize(self, vsize: VSize) -> VSize {
        vsize.max(VSize::new(self.0 * Self::VBYTES_PER_SIGOP))
    }
}

/// Narrowing to the storage form: a transaction's sigop cost fits 32 bits (checked in dev builds).
impl From<SigOps> for SigOps32 {
    #[inline]
    fn from(value: SigOps) -> Self {
        debug_assert!(
            u32::try_from(value.0).is_ok(),
            "{} doesn't fit SigOps32",
            value.0
        );
        Self::new(value.0 as u32)
    }
}

impl From<u64> for SigOps {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<SigOps> for u64 {
    #[inline]
    fn from(value: SigOps) -> Self {
        value.0
    }
}

impl From<SigOps32> for SigOps {
    #[inline]
    fn from(value: SigOps32) -> Self {
        Self(u64::from(u32::from(value)))
    }
}

impl From<usize> for SigOps {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

/// Saturating (`as` from a float clamps and maps NaN to zero).
impl From<f64> for SigOps {
    #[inline]
    fn from(value: f64) -> Self {
        Self(value as u64)
    }
}

impl From<SigOps> for f64 {
    #[inline]
    fn from(value: SigOps) -> Self {
        value.0 as f64
    }
}

impl Add for SigOps {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for SigOps {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for SigOps {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for SigOps {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Sum for SigOps {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|value| value.0).sum())
    }
}

impl CheckedSub for SigOps {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

#[cfg(feature = "storage")]
impl VecdbCheckedSub for SigOps {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl Display for SigOps {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(Buffer::new().format(self.0))
    }
}

#[cfg(feature = "storage")]
impl Formattable for SigOps {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(Buffer::new().format(self.0).as_bytes());
    }
}
