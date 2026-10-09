use bitview_primitives::Date;
use brk_types::{Bitcoin, BlockHash, Height};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The UTXO set after a block, grouped by the height of the block that created each output.
///
/// The set excludes `OP_RETURN` outputs and the genesis coinbase, which are unspendable, and the
/// coinbases of blocks 91812 and 91722 once duplicates at 91842 and 91880 overwrite them; it
/// includes zero-value outputs. The set after block `first - 1` plus a diff's `created` minus its
/// `spent` gives the set after block `last`; rounding each supply to 8 decimals keeps it exact.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UtxoSet {
    /// Height of the represented block.
    pub height: Height,
    /// Hash of the represented block.
    pub hash: BlockHash,
    /// UTC date of the represented block's monotonic timestamp (the running maximum of block
    /// times).
    pub date: Date,
    /// Number of unspent outputs.
    pub count: u64,
    /// BTC held by the unspent outputs.
    pub supply: Bitcoin,
    /// Unspent outputs by creation height: entry `i` is what remains of block `i`'s outputs.
    pub origins: UtxoOrigins,
}

/// Columnar unspent amounts indexed by creation height.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
pub struct UtxoOrigins {
    /// Number of unspent outputs per creation height.
    pub count: Vec<u64>,
    /// BTC held by the unspent outputs per creation height.
    pub supply: Vec<Bitcoin>,
}

/// What blocks `first` through `last` changed in the UTXO set.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UtxoSetDiff {
    /// First block of the diff; it applies to the set after block `first - 1` (the empty set for
    /// block 0).
    pub first: Height,
    /// Last block of the diff, described by `hash` and `date`.
    pub last: Height,
    /// Hash of block `last`.
    pub hash: BlockHash,
    /// UTC date of block `last`'s monotonic timestamp.
    pub date: Date,
    /// Outputs created, one row per block, including those spent again within the range (which
    /// `spent` lists too).
    pub created: UtxoChanges,
    /// Outputs removed, one row per creation height in ascending order: spent outputs, plus the
    /// coinbase outputs of blocks 91812 and 91722, which duplicates at 91842 and 91880 overwrote.
    pub spent: UtxoChanges,
}

/// Columnar counts and supplies by creation height.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
pub struct UtxoChanges {
    /// Creation height of each row.
    pub height: Vec<Height>,
    /// Number of outputs in each row.
    pub count: Vec<u64>,
    /// BTC held by the outputs in each row.
    pub supply: Vec<Bitcoin>,
}
