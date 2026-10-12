use bitview_primitives::{TxOutIndex, TypeIndex};
use brk_types::{OutputType, Sats, StoreValue, TxIndex, TxidPrefix, Vout};
use vecdb::{Bytes, OverflowVecValue};

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

/// A large value's marker in the entry: its value is in the overflow store.
const LARGE: u32 = u32::MAX;

/// What resolving an input needs about the output it spends, packed (18 bytes, no padding: the pending map holds
/// millions of them between saves). The output index takes 5 bytes (below 2^40, as `Index40`); the value is
/// `Sats`' compact form, as in `outputs.value`, and a large one (2^31 sats or more) is kept in the overflow store.
#[derive(Debug, Clone, Copy)]
pub struct Utxo([u8; 18]);

impl Utxo {
    #[inline]
    pub(crate) fn new(
        tx_index: TxIndex,
        txout_index: TxOutIndex,
        output_type: OutputType,
        type_index: TypeIndex,
        value: Sats,
    ) -> Self {
        let txout_index = u64::from(txout_index);
        debug_assert!(txout_index < 1 << 40, "output index above 2^40");
        let mut bytes = [0; 18];
        bytes[..4].copy_from_slice(&u32::from(tx_index).to_be_bytes());
        bytes[4..9].copy_from_slice(&txout_index.to_be_bytes()[3..]);
        bytes[9] = output_type as u8;
        bytes[10..14].copy_from_slice(&u32::from(type_index).to_be_bytes());
        bytes[14..].copy_from_slice(&value.to_compact().unwrap_or(LARGE).to_be_bytes());
        Self(bytes)
    }

    /// Whether `value` is kept in the overflow store instead of the entry.
    #[inline]
    pub(crate) fn is_large(value: Sats) -> bool {
        value.to_compact().is_none()
    }

    #[inline]
    pub fn tx_index(&self) -> TxIndex {
        TxIndex::from(u32::from_be_bytes(self.0[..4].try_into().unwrap()))
    }

    #[inline]
    pub fn txout_index(&self) -> TxOutIndex {
        let mut bytes = [0; 8];
        bytes[3..].copy_from_slice(&self.0[4..9]);
        TxOutIndex::from(u64::from_be_bytes(bytes))
    }

    #[inline]
    pub fn output_type(&self) -> OutputType {
        OutputType::from_bytes(&self.0[9..10]).expect("stored output type")
    }

    #[inline]
    pub fn type_index(&self) -> TypeIndex {
        TypeIndex::from(u32::from_be_bytes(self.0[10..14].try_into().unwrap()))
    }

    /// The output's value, or `None` for a large one, which is in the overflow store.
    #[inline]
    pub fn value(&self) -> Option<Sats> {
        let compact = u32::from_be_bytes(self.0[14..].try_into().unwrap());
        (compact != LARGE).then(|| Sats::from_compact(compact))
    }
}

impl StoreValue for Utxo {
    type Bytes = [u8; 18];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(bytes)
    }
}
