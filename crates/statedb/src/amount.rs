use crate::util::invalid;
use std::{
    io::Result,
    ops::{Add, Sub},
};
use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, FromBytes, Immutable, IntoBytes)]
pub struct Amount {
    pub sats: u64,
    pub count: u64,
}
impl Amount {
    pub(crate) fn checked_add(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            sats: self
                .sats
                .checked_add(rhs.sats)
                .ok_or_else(|| invalid("supply overflow"))?,
            count: self
                .count
                .checked_add(rhs.count)
                .ok_or_else(|| invalid("count overflow"))?,
        })
    }
    pub(crate) fn encode(self) -> [u8; 16] {
        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&self.sats.to_le_bytes());
        bytes[8..].copy_from_slice(&self.count.to_le_bytes());
        bytes
    }
    /// `bytes` is a 16-byte field of a record whose framing was checked.
    pub(crate) fn decode(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), 16);
        Self {
            sats: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            count: u64::from_le_bytes(bytes[8..].try_into().unwrap()),
        }
    }
}
/// Replay arithmetic over validated rows: overflow is a debug-checked invariant.
impl Add for Amount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            sats: self.sats + rhs.sats,
            count: self.count + rhs.count,
        }
    }
}
impl Sub for Amount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            sats: self.sats - rhs.sats,
            count: self.count - rhs.count,
        }
    }
}
