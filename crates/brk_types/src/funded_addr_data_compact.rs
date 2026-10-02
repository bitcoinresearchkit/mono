#[cfg(feature = "storage")]
use vecdb::Result;

#[cfg(feature = "storage")]
use vecdb::Bytes;

const COUNT_BITS: u32 = 21;
const COUNT_MASK: u32 = (1 << COUNT_BITS) - 1;
const OVERFLOW_TAG: u64 = 1 << 63;

/// Compact inline storage used by `OverflowVec<_, FundedAddrData>`.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FundedAddrDataCompact {
    received: u64,
    sent: u64,
    realized_cap_raw: u64,
    counts: u64,
}

impl FundedAddrDataCompact {
    #[inline(always)]
    pub(crate) fn new(
        received: u64,
        sent: u64,
        realized_cap_raw: u128,
        tx_count: u32,
        funded_txo_count: u32,
        spent_txo_count: u32,
    ) -> Option<Self> {
        if realized_cap_raw > u128::from(u64::MAX)
            || tx_count > COUNT_MASK
            || funded_txo_count > COUNT_MASK
            || spent_txo_count > COUNT_MASK
        {
            return None;
        }

        Some(Self {
            received,
            sent,
            realized_cap_raw: realized_cap_raw as u64,
            counts: u64::from(tx_count)
                | (u64::from(funded_txo_count) << COUNT_BITS)
                | (u64::from(spent_txo_count) << (COUNT_BITS * 2)),
        })
    }

    #[inline(always)]
    pub(crate) fn received(self) -> u64 {
        self.received
    }

    #[inline(always)]
    pub(crate) fn sent(self) -> u64 {
        self.sent
    }

    #[inline(always)]
    pub(crate) fn realized_cap_raw(self) -> u64 {
        self.realized_cap_raw
    }

    #[inline(always)]
    pub(crate) fn tx_count(self) -> u32 {
        (self.counts & u64::from(COUNT_MASK)) as u32
    }

    #[inline(always)]
    pub(crate) fn funded_txo_count(self) -> u32 {
        ((self.counts >> COUNT_BITS) & u64::from(COUNT_MASK)) as u32
    }

    #[inline(always)]
    pub(crate) fn spent_txo_count(self) -> u32 {
        ((self.counts >> (COUNT_BITS * 2)) & u64::from(COUNT_MASK)) as u32
    }

    #[inline(always)]
    pub(crate) fn overflow_index(self) -> Option<usize> {
        (self.counts & OVERFLOW_TAG != 0)
            .then(|| usize::try_from(self.received).expect("overflow index must fit usize"))
    }

    #[inline(always)]
    pub(crate) fn from_overflow_index(index: usize) -> Self {
        Self {
            received: u64::try_from(index).expect("overflow index must fit u64"),
            sent: 0,
            realized_cap_raw: 0,
            counts: OVERFLOW_TAG,
        }
    }
}

#[cfg(feature = "storage")]
impl Bytes for FundedAddrDataCompact {
    type Array = [u8; 32];

    const IS_NATIVE_LAYOUT: bool = cfg!(target_endian = "little");

    fn to_bytes(&self) -> Self::Array {
        let mut bytes = [0; 32];
        bytes[0..8].copy_from_slice(&self.received.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.sent.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.realized_cap_raw.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.counts.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            received: u64::from_bytes(&bytes[0..8])?,
            sent: u64::from_bytes(&bytes[8..16])?,
            realized_cap_raw: u64::from_bytes(&bytes[16..24])?,
            counts: u64::from_bytes(&bytes[24..32])?,
        })
    }
}
