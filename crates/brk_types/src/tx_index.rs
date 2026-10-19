use derive_more::{Deref, DerefMut};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

    pub fn index_name() -> &'static str {
        "tx_index"
    }

    pub fn index_aliases() -> &'static [&'static str] {
        &["tx", "tx_index"]
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

crate::impl_store_value!(TxIndex, u32);

#[cfg(feature = "storage")]
impl PrintableIndex for TxIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxIndex {
    const INITIAL_CAPACITY: usize = 1_700_000_000;
}
