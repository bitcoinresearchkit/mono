use crate::util::invalid;
use std::io::Result;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Amount {
    pub sats: u64,
    pub count: u64,
}
impl Amount {
    pub fn checked_add(self, rhs: Self) -> Result<Self> {
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
    pub fn checked_sub(self, rhs: Self) -> Result<Self> {
        Ok(Self {
            sats: self
                .sats
                .checked_sub(rhs.sats)
                .ok_or_else(|| invalid("supply underflow"))?,
            count: self
                .count
                .checked_sub(rhs.count)
                .ok_or_else(|| invalid("count underflow"))?,
        })
    }
    pub(crate) fn encode(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.sats.to_le_bytes());
        out.extend_from_slice(&self.count.to_le_bytes());
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 16 {
            return Err(invalid("invalid amount size"));
        }
        Ok(Self {
            sats: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            count: u64::from_le_bytes(bytes[8..].try_into().unwrap()),
        })
    }
}
