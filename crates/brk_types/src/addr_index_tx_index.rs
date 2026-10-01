use std::hash::Hash;

use serde::Serialize;
#[cfg(feature = "storage")]
use vecdb::Bytes;

use super::{TxIndex, TypeIndex};
use crate::StoreValue;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
#[cfg_attr(feature = "storage", derive(Bytes))]
pub struct AddrIndexTxIndex(u64);

impl AddrIndexTxIndex {
    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(self.0 as u32)
    }

    pub fn min_for_addr(addr_index: TypeIndex) -> Self {
        Self(u64::from(addr_index) << 32)
    }
}

impl From<(TypeIndex, TxIndex)> for AddrIndexTxIndex {
    #[inline]
    fn from((addr_index, tx_index): (TypeIndex, TxIndex)) -> Self {
        Self((u64::from(addr_index) << 32) | u64::from(tx_index))
    }
}

impl StoreValue for AddrIndexTxIndex {
    type Bytes = [u8; 8];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(u64::from_be_bytes(bytes))
    }
}
