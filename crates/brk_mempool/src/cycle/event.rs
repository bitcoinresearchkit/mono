use std::{sync::Arc, time::Duration};

use brk_types::{AddressBytes, BlockHash, Height, MempoolInfo};

use crate::{
    Snapshot,
    cycle::{TxAdded, TxRemoved},
};

/// One pull cycle's worth of changes. Produced by
/// [`crate::Mempool::tick`] after fetch → prepare → apply → prevouts →
/// rebuild. These are private working-cycle events, including applied changes
/// when the final chain observation fails. The graph supplies the next cycle's
/// burial rates; it is not proof of a public commit. Use a read-only handle for
/// the last valid published data.
pub struct Cycle {
    pub added: Vec<TxAdded>,
    pub removed: Vec<TxRemoved>,
    /// Addresses that went from 0 → 1+ live mempool txs this cycle.
    /// Same-cycle enter-then-leave is collapsed (no event in either list).
    pub addr_enters: Vec<AddressBytes>,
    /// Addresses that went from 1+ → 0 live mempool txs this cycle.
    pub addr_leaves: Vec<AddressBytes>,
    /// Latest confirmed block. Compare to the prior cycle's `tip_hash`
    /// to detect a new block.
    pub tip_hash: BlockHash,
    pub tip_height: Height,
    pub info: MempoolInfo,
    pub snapshot: Arc<Snapshot>,
    pub took: Duration,
}
