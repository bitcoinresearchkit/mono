use std::hash::Hash;

use byteview::ByteView;
use serde::Serialize;
use vecdb::Bytes;

use super::{TxIndex, TypeIndex};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Bytes, Hash)]
pub struct AddrIndexTxIndex(u64);

impl AddrIndexTxIndex {
    pub fn addr_index(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(self.0 as u32)
    }

    pub fn min_for_addr(addr_index: TypeIndex) -> Self {
        Self(u64::from(addr_index) << 32)
    }

    pub fn max_for_addr(addr_index: TypeIndex) -> Self {
        Self((u64::from(addr_index) << 32) | u64::MAX >> 32)
    }
}

impl From<(TypeIndex, TxIndex)> for AddrIndexTxIndex {
    #[inline]
    fn from((addr_index, tx_index): (TypeIndex, TxIndex)) -> Self {
        Self((u64::from(addr_index) << 32) | u64::from(tx_index))
    }
}

impl From<ByteView> for AddrIndexTxIndex {
    #[inline]
    fn from(value: ByteView) -> Self {
        Self(u64::from_be_bytes((&*value).try_into().unwrap()))
    }
}

impl From<AddrIndexTxIndex> for ByteView {
    #[inline]
    fn from(value: AddrIndexTxIndex) -> Self {
        ByteView::from(&value)
    }
}
impl From<&AddrIndexTxIndex> for ByteView {
    #[inline]
    fn from(value: &AddrIndexTxIndex) -> Self {
        ByteView::from(value.0.to_be_bytes())
    }
}
