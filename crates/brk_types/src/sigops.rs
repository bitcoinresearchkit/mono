use derive_more::{Add, AddAssign, Deref, From, Into, Sub, SubAssign, Sum};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, SigOps32, VSize};

#[cfg(feature = "storage")]
use vecdb::Pco;

/// BIP-141 signature-operation cost: legacy and P2SH-redeem sigops count four and witness sigops
/// one. Sums over many transactions or blocks use the full 64 bits.
#[derive(
    Debug,
    Default,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Sum,
    From,
    Into,
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

impl CheckedSub for SigOps {
    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

crate::impl_int_format!(SigOps);
