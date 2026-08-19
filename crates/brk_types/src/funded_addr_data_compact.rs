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
    pub fn new(
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
    pub fn received(self) -> u64 {
        self.received
    }

    #[inline(always)]
    pub fn sent(self) -> u64 {
        self.sent
    }

    #[inline(always)]
    pub fn realized_cap_raw(self) -> u64 {
        self.realized_cap_raw
    }

    #[inline(always)]
    pub fn tx_count(self) -> u32 {
        (self.counts & u64::from(COUNT_MASK)) as u32
    }

    #[inline(always)]
    pub fn funded_txo_count(self) -> u32 {
        ((self.counts >> COUNT_BITS) & u64::from(COUNT_MASK)) as u32
    }

    #[inline(always)]
    pub fn spent_txo_count(self) -> u32 {
        ((self.counts >> (COUNT_BITS * 2)) & u64::from(COUNT_MASK)) as u32
    }

    #[inline(always)]
    pub fn overflow_index(self) -> Option<usize> {
        (self.counts & OVERFLOW_TAG != 0)
            .then(|| usize::try_from(self.received).expect("overflow index must fit usize"))
    }

    #[inline(always)]
    pub fn from_overflow_index(index: usize) -> Self {
        Self {
            received: u64::try_from(index).expect("overflow index must fit u64"),
            sent: 0,
            realized_cap_raw: 0,
            counts: OVERFLOW_TAG,
        }
    }
}

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

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        Ok(Self {
            received: u64::from_bytes(&bytes[0..8])?,
            sent: u64::from_bytes(&bytes[8..16])?,
            realized_cap_raw: u64::from_bytes(&bytes[16..24])?,
            counts: u64::from_bytes(&bytes[24..32])?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_and_overflow_encodings_are_disjoint() {
        assert_eq!(size_of::<FundedAddrDataCompact>(), 32);
        let inline = FundedAddrDataCompact::new(
            u64::MAX,
            u64::MAX,
            u128::from(u64::MAX),
            COUNT_MASK,
            COUNT_MASK,
            COUNT_MASK,
        )
        .unwrap();
        assert_eq!(inline.overflow_index(), None);
        assert_eq!(inline.received(), u64::MAX);
        assert_eq!(inline.sent(), u64::MAX);
        assert_eq!(inline.realized_cap_raw(), u64::MAX);
        assert_eq!(inline.tx_count(), COUNT_MASK);
        assert_eq!(inline.funded_txo_count(), COUNT_MASK);
        assert_eq!(inline.spent_txo_count(), COUNT_MASK);

        for index in [0, 1, 42, 1_000_000] {
            let pointer = FundedAddrDataCompact::from_overflow_index(index);
            assert_eq!(pointer.overflow_index(), Some(index));
        }
    }
}
