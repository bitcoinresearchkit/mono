use brk_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use crate::internal::PerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// BIP-141 signature-operation cost. At `tx_index`, this is the cost of the
    /// indexed transaction. At `height`, this is the sum across every
    /// transaction in the block, including coinbase. Sigops in legacy
    /// scriptPubKeys, scriptSigs, and P2SH redeemScripts cost four units;
    /// P2WPKH and P2WSH sigops cost one. This statically counts
    /// signature-checking operations rather than signatures actually executed.
    /// Tapscript signature opcodes are excluded because BIP-342 uses a separate
    /// per-input execution budget. The post-SegWit consensus block limit is
    /// 80,000 cost units.
    pub total: PerBlockCumulativeRolling<StoredU64, M>,
}
