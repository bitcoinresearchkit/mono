use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{Bytes, Formattable, OverflowVecValue, Version};

use crate::{FundedAddrData, Sats};

/// Data of an empty address
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[repr(C)]
pub struct EmptyAddrData {
    /// Total transaction count
    pub tx_count: u32,
    /// Total funded/spent transaction output count (equal since address is empty)
    pub funded_txo_count: u32,
    /// Total satoshis transferred
    pub transfered: Sats,
}

impl From<FundedAddrData> for EmptyAddrData {
    #[inline]
    fn from(value: FundedAddrData) -> Self {
        Self::from(&value)
    }
}

impl From<&FundedAddrData> for EmptyAddrData {
    #[inline]
    fn from(value: &FundedAddrData) -> Self {
        if value.sent != value.received {
            dbg!(&value);
            panic!("Trying to convert not empty wallet to empty !");
        }
        Self {
            tx_count: value.tx_count,
            funded_txo_count: value.funded_txo_count,
            transfered: value.sent,
        }
    }
}

impl std::fmt::Display for EmptyAddrData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "tx_count: {}, funded_txo_count: {}, transfered: {}",
            self.tx_count, self.funded_txo_count, self.transfered
        )
    }
}

impl Formattable for EmptyAddrData {
    fn write_to(&self, buf: &mut Vec<u8>) {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}", self).unwrap();
        buf.extend_from_slice(s.as_bytes());
    }

    fn fmt_csv(&self, f: &mut String) -> std::fmt::Result {
        let start = f.len();
        self.fmt_into(f);
        if f.as_bytes()[start..].contains(&b',') {
            f.insert(start, '"');
            f.push('"');
        }
        Ok(())
    }

    fn fmt_json(&self, buf: &mut Vec<u8>) {
        buf.push(b'"');
        self.write_to(buf);
        buf.push(b'"');
    }
}

impl Bytes for EmptyAddrData {
    type Array = [u8; size_of::<Self>()];

    fn to_bytes(&self) -> Self::Array {
        let mut arr = [0u8; size_of::<Self>()];
        arr[0..4].copy_from_slice(self.tx_count.to_bytes().as_ref());
        arr[4..8].copy_from_slice(self.funded_txo_count.to_bytes().as_ref());
        arr[8..16].copy_from_slice(self.transfered.to_bytes().as_ref());
        arr
    }

    fn from_bytes(bytes: &[u8]) -> vecdb::Result<Self> {
        Ok(Self {
            tx_count: u32::from_bytes(&bytes[0..4])?,
            funded_txo_count: u32::from_bytes(&bytes[4..8])?,
            transfered: Sats::from_bytes(&bytes[8..16])?,
        })
    }
}

impl OverflowVecValue for EmptyAddrData {
    type Compact = u64;

    const VERSION: Version = Version::ONE;

    #[inline(always)]
    fn to_compact(&self) -> Option<Self::Compact> {
        const COUNT_MASK: u64 = (1 << 12) - 1;
        const TRANSFER_MASK: u64 = (1 << 40) - 1;

        let tx_count = u64::from(self.tx_count);
        let funded_txo_count = u64::from(self.funded_txo_count);
        let transfered = u64::from(self.transfered);

        if tx_count == 0 {
            return (funded_txo_count == 0 && transfered == 0).then_some(0);
        }
        if tx_count > COUNT_MASK || funded_txo_count > COUNT_MASK || transfered > TRANSFER_MASK {
            return None;
        }

        Some(tx_count | (funded_txo_count << 12) | (transfered << 24))
    }

    #[inline(always)]
    fn from_compact(compact: Self::Compact) -> Self {
        const COUNT_MASK: u64 = (1 << 12) - 1;
        const TRANSFER_MASK: u64 = (1 << 40) - 1;

        debug_assert!(Self::overflow_index(compact).is_none());
        Self {
            tx_count: (compact & COUNT_MASK) as u32,
            funded_txo_count: ((compact >> 12) & COUNT_MASK) as u32,
            transfered: Sats::from((compact >> 24) & TRANSFER_MASK),
        }
    }

    #[inline(always)]
    fn overflow_index(compact: Self::Compact) -> Option<usize> {
        const COUNT_MASK: u64 = (1 << 12) - 1;

        (compact != 0 && compact & COUNT_MASK == 0)
            .then(|| usize::try_from((compact >> 12) - 1).expect("overflow index fits usize"))
    }

    #[inline(always)]
    fn from_overflow_index(index: usize) -> Self::Compact {
        const MAX_POINTER: u64 = (1 << 52) - 1;

        let pointer = u64::try_from(index)
            .expect("overflow index fits u64")
            .checked_add(1)
            .expect("overflow index must leave room for its tag");
        assert!(
            pointer <= MAX_POINTER,
            "overflow sidecar index is too large"
        );
        pointer << 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_roundtrip_and_boundaries() {
        let max_inline = EmptyAddrData {
            tx_count: (1 << 12) - 1,
            funded_txo_count: (1 << 12) - 1,
            transfered: Sats::from((1_u64 << 40) - 1),
        };
        let compact = max_inline.to_compact().unwrap();
        let decoded = EmptyAddrData::from_compact(compact);
        assert_eq!(decoded.tx_count, max_inline.tx_count);
        assert_eq!(decoded.funded_txo_count, max_inline.funded_txo_count);
        assert_eq!(decoded.transfered, max_inline.transfered);

        assert!(
            EmptyAddrData {
                tx_count: 1 << 12,
                ..Default::default()
            }
            .to_compact()
            .is_none()
        );
        assert!(
            EmptyAddrData {
                tx_count: 1,
                funded_txo_count: 1 << 12,
                ..Default::default()
            }
            .to_compact()
            .is_none()
        );
        assert!(
            EmptyAddrData {
                tx_count: 1,
                transfered: Sats::from(1_u64 << 40),
                ..Default::default()
            }
            .to_compact()
            .is_none()
        );
    }

    #[test]
    fn default_and_overflow_tags_are_unambiguous() {
        let default = EmptyAddrData::default().to_compact().unwrap();
        assert_eq!(default, 0);
        assert_eq!(EmptyAddrData::overflow_index(default), None);

        for index in [0, 1, 42, 1_000_000] {
            let compact = EmptyAddrData::from_overflow_index(index);
            assert_eq!(EmptyAddrData::overflow_index(compact), Some(index));
        }
    }
}
