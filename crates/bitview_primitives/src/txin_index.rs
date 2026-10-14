use std::ops::Add;

use brk_types::Vin;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex, VecIndex};

/// Index of a transaction input.
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
    JsonSchema,
)]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct TxInIndex(u64);

impl TxInIndex {
    /// Sentinel value indicating an unspent output.
    pub const UNSPENT: Self = Self(u64::MAX);

    pub fn new(index: u64) -> Self {
        Self(index)
    }

    pub fn is_unspent(self) -> bool {
        self == Self::UNSPENT
    }
}

brk_types::impl_integer_index!(TxInIndex, u64);

impl Add<Vin> for TxInIndex {
    type Output = Self;
    fn add(self, rhs: Vin) -> Self::Output {
        Self(self.0 + u64::from(rhs))
    }
}

impl TxInIndex {
    pub(crate) fn index_name() -> &'static str {
        "txin_index"
    }
    pub(crate) fn index_aliases() -> &'static [&'static str] {
        &["txi", "txin", "txin_index"]
    }
}
#[cfg(feature = "storage")]
impl PrintableIndex for TxInIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
    fn to_possible_strings() -> &'static [&'static str] {
        Self::index_aliases()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxInIndex {
    const INITIAL_CAPACITY: usize = 4_200_000_000;
}
