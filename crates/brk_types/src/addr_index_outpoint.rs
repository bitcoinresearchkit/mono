use std::mem;

use serde::Serialize;

use super::{OutPoint, TxIndex, TypeIndex, Vout};
use crate::StoreValue;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
pub struct AddrIndexOutPoint([u8; 10]);

const _: () = assert!(mem::size_of::<AddrIndexOutPoint>() == 10);

impl AddrIndexOutPoint {
    #[inline]
    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(u32::from_be_bytes([
            self.0[4], self.0[5], self.0[6], self.0[7],
        ]))
    }

    #[inline]
    pub fn vout(&self) -> Vout {
        Vout::from(u16::from_be_bytes([self.0[8], self.0[9]]))
    }
}

impl From<(TypeIndex, OutPoint)> for AddrIndexOutPoint {
    #[inline]
    fn from((addr_index, outpoint): (TypeIndex, OutPoint)) -> Self {
        let mut bytes = [0; 10];
        bytes[..4].copy_from_slice(&u32::from(addr_index).to_be_bytes());
        bytes[4..8].copy_from_slice(&u32::from(outpoint.tx_index()).to_be_bytes());
        bytes[8..].copy_from_slice(&outpoint.vout().to_be_bytes());
        Self(bytes)
    }
}

impl StoreValue for AddrIndexOutPoint {
    type Bytes = [u8; 10];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_encoding_is_stable_and_roundtrips() {
        let value = AddrIndexOutPoint::from((
            TypeIndex::new(0x0102_0304),
            OutPoint::new(TxIndex::new(0x0506_0708), Vout::from(0x090a_u16)),
        ));
        let bytes = value.to_store_bytes();

        assert_eq!(
            &bytes,
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "the LSM key encoding is part of the persisted format",
        );
        assert_eq!(AddrIndexOutPoint::from_store_bytes(bytes), value);
        assert_eq!(value.tx_index(), TxIndex::new(0x0506_0708));
        assert_eq!(value.vout(), Vout::from(0x090a_u16));
    }
}
