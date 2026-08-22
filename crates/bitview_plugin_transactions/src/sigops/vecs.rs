use bitview_traversable::Traversable;
use brk_types::StoredU64;
use vecdb::{Rw, StorageMode};

use bitview_compute::PerBlockCumulativeRolling;

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// BIP-141 signature-operation cost. At `tx_index`, this is the indexed
    /// transaction's cost; at `height`, it is the block total including
    /// coinbase. Legacy scriptPubKey, scriptSig, and P2SH redeem-script sigops
    /// cost four units; P2WPKH and P2WSH sigops cost one. This is a static count,
    /// not the number of signatures executed. Tapscript sigops are excluded
    /// because BIP-342 uses a separate per-input budget. The post-SegWit block
    /// limit is 80,000 cost units.
    pub total: PerBlockCumulativeRolling<StoredU64, M>,
}
