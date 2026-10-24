use std::ops::Add;

use brk_types::Vout;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use vecdb::{Pco, PrintableIndex, VecIndex};

/// Index of a transaction output.
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
pub struct TxOutIndex(u64);

impl TxOutIndex {
    pub const ZERO: Self = Self(0);

    pub const COINBASE: Self = Self(u64::MAX);

    pub fn is_coinbase(self) -> bool {
        self == Self::COINBASE
    }

    pub(crate) fn index_name() -> &'static str {
        "txout_index"
    }
}

brk_types::impl_integer_index!(TxOutIndex, u64);

impl Add<Vout> for TxOutIndex {
    type Output = Self;
    fn add(self, rhs: Vout) -> Self::Output {
        Self(self.0 + u64::from(rhs))
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for TxOutIndex {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for TxOutIndex {
    const INITIAL_CAPACITY: usize = 4_700_000_000;
}
