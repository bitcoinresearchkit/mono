use derive_more::Deref;

use super::Txid;
use crate::StoreValue;

/// First-8-bytes prefix of a txid, packed as a `u64`. Both `From<&Txid>`
/// (via `from_le_bytes`) and `StoreValue` (via `from_be_bytes`,
/// inverse of the `to_be_bytes` writer) are host-independent so on-disk
/// keys are portable across architectures.
#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TxidPrefix(u64);

impl From<Txid> for TxidPrefix {
    #[inline]
    fn from(value: Txid) -> Self {
        Self::from(&value)
    }
}

impl From<&Txid> for TxidPrefix {
    #[inline]
    fn from(value: &Txid) -> Self {
        Self(u64::from_le_bytes(
            value.as_slice()[0..8].try_into().unwrap(),
        ))
    }
}

impl StoreValue for TxidPrefix {
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
