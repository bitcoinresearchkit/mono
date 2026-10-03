#[cfg(feature = "storage")]
use itoa::Buffer;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::VSize;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// BIP-141 sigop cost. The block-level budget is 80,000, so a `u32`
/// fits a single tx's count with room to spare.
///
/// Witness sigops count as 1; legacy and P2SH-redeem sigops count as 4.
/// Five vbytes per sigop is the policy adjustment Core applies in
/// `nSigOpCost` to discourage sigop-heavy txs (`max(weight/4, sigops*5)`).
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[serde(transparent)]
pub struct SigOps(u32);

impl SigOps {
    pub const ZERO: Self = Self(0);

    /// Vbytes per sigop under BIP-141 policy. Core's `nSigOpCost`
    /// adjustment factor: `adjusted_vsize = max(vsize, sigops * 5)`.
    const VBYTES_PER_SIGOP: u64 = 5;

    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// BIP-141 vbyte equivalent of this sigop count.
    #[inline]
    fn vsize_cost(self) -> VSize {
        VSize::new(u64::from(self.0) * Self::VBYTES_PER_SIGOP)
    }

    /// Policy-adjusted vsize: `max(vsize, sigops * 5)`. The denominator
    /// Core uses when ranking sigop-heavy txs at fixed fee.
    #[inline]
    pub fn adjust_vsize(self, vsize: VSize) -> VSize {
        vsize.max(self.vsize_cost())
    }
}

impl From<u32> for SigOps {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<usize> for SigOps {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl From<SigOps> for u32 {
    #[inline]
    fn from(value: SigOps) -> Self {
        value.0
    }
}

#[cfg(feature = "storage")]
impl Formattable for SigOps {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
