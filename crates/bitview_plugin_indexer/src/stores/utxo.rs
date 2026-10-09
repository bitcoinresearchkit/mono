use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_types::{OutputType, StoreValue, TxIndex, TxidPrefix, Vout};
use vecdb::Bytes;

/// Key of an unspent output: the creating transaction's txid prefix, then the output's vout.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct UtxoKey([u8; 10]);

impl UtxoKey {
    #[inline]
    pub fn new(txid_prefix: TxidPrefix, vout: Vout) -> Self {
        let mut bytes = [0; 10];
        bytes[..8].copy_from_slice(&txid_prefix.to_store_bytes());
        bytes[8..].copy_from_slice(&vout.to_be_bytes());
        Self(bytes)
    }
}

impl StoreValue for UtxoKey {
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

/// What resolving an input needs about the output it spends, packed (17 bytes, no padding: the pending map
/// holds millions of them between saves).
#[derive(Debug, Clone, Copy)]
pub struct Utxo([u8; 17]);

impl Utxo {
    #[inline]
    pub(crate) fn new(
        tx_index: TxIndex,
        txout_index: TxOutIndex,
        output_type: OutputType,
        type_index: TypeIndex,
    ) -> Self {
        let mut bytes = [0; 17];
        bytes[..4].copy_from_slice(&u32::from(tx_index).to_be_bytes());
        bytes[4..12].copy_from_slice(&u64::from(txout_index).to_be_bytes());
        bytes[12] = output_type as u8;
        bytes[13..].copy_from_slice(&u32::from(type_index).to_be_bytes());
        Self(bytes)
    }

    #[inline]
    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(u32::from_be_bytes(self.0[..4].try_into().unwrap()))
    }

    #[inline]
    pub fn txout_index(&self) -> TxOutIndex {
        TxOutIndex::from(u64::from_be_bytes(self.0[4..12].try_into().unwrap()))
    }

    #[inline]
    pub fn output_type(&self) -> OutputType {
        OutputType::from_bytes(&self.0[12..13]).expect("stored output type")
    }

    #[inline]
    pub fn type_index(&self) -> TypeIndex {
        TypeIndex::from(u32::from_be_bytes(self.0[13..].try_into().unwrap()))
    }
}

impl StoreValue for Utxo {
    type Bytes = [u8; 17];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(bytes)
    }
}
