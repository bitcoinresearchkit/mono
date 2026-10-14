use derive_more::{Deref, DerefMut};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::StoreValue;

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex, VecIndex};

/// Chain-wide transaction index (0 = the genesis coinbase). For an
/// in-block position, use `BlockTxIndex` instead.
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Deref,
    DerefMut,
    Default,
    Serialize,
    Deserialize,
    Hash,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[cfg_attr(feature = "schemars", schemars(example = 0))]
pub struct TxIndex(u32);

impl TxIndex {
    pub const ZERO: Self = Self(0);
    pub const fn new(tx_index: u32) -> Self {
        Self(tx_index)
    }

    pub fn incremented(self) -> Self {
        Self(*self + 1)
    }
}

crate::impl_integer_index!(TxIndex, u32);

impl From<u64> for TxIndex {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}
impl From<TxIndex> for u64 {
    #[inline]
    fn from(value: TxIndex) -> Self {
        value.0 as u64
    }
}

impl StoreValue for TxIndex {
    type Bytes = [u8; 4];

    #[inline]
    fn to_store_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()
    }

    #[inline]
    fn from_store_bytes(bytes: Self::Bytes) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
}

impl TxIndex {
    pub fn index_name() -> &'static str {
        "tx_index"
    }
    pub fn index_aliases() -> &'static [&'static str] {
        &["tx", "tx_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for TxIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxIndex {
    const INITIAL_CAPACITY: usize = 1_700_000_000;
}
