use std::fmt::{Display, Formatter, Result};

use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Position of a transaction within a single block (0 = coinbase).
/// Distinct from `TxIndex`, which is the chain-wide global tx index.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Serialize, Deserialize, JsonSchema,
)]
#[schemars(example = 0)]
pub struct BlockTxIndex(u32);

impl From<u32> for BlockTxIndex {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<BlockTxIndex> for u32 {
    #[inline]
    fn from(value: BlockTxIndex) -> Self {
        value.0
    }
}

impl From<BlockTxIndex> for usize {
    #[inline]
    fn from(value: BlockTxIndex) -> Self {
        value.0 as usize
    }
}

impl Display for BlockTxIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buf = Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}
