use std::hash::Hash;

use bitview_primitives::TypeIndex;
use brk_types::{OutPoint, TxIndex, Vout};
use serde::Serialize;
use vecdb::Bytes;

/// An address's unspent output: the address, the transaction, the output, each big-endian so a
/// prefix walk lists an address's outputs by transaction then vout. The vout in full: a transaction
/// may hold more than 65,536 outputs to one address (consensus allows it), and a narrower one would
/// file two of them under one key.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash)]
pub struct AddrIndexOutPoint([u8; 12]);

impl AddrIndexOutPoint {
    #[inline]
    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(u32::from_be_bytes([
            self.0[4], self.0[5], self.0[6], self.0[7],
        ]))
    }

    #[inline]
    pub fn vout(&self) -> Vout {
        Vout::from(u32::from_be_bytes([
            self.0[8], self.0[9], self.0[10], self.0[11],
        ]))
    }
}

impl From<(TypeIndex, OutPoint)> for AddrIndexOutPoint {
    #[inline]
    fn from((addr_index, outpoint): (TypeIndex, OutPoint)) -> Self {
        let mut bytes = [0; 12];
        bytes[..4].copy_from_slice(&u32::from(addr_index).to_be_bytes());
        bytes[4..8].copy_from_slice(&u32::from(outpoint.tx_index()).to_be_bytes());
        bytes[8..].copy_from_slice(&u32::from(outpoint.vout()).to_be_bytes());
        Self(bytes)
    }
}

brk_types::impl_store_value!(AddrIndexOutPoint, [u8; 12]);

/// A transaction an address took part in: the address in the high half, the transaction in the low.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Hash, Bytes)]
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

brk_types::impl_store_value!(AddrIndexTxIndex, u64);
