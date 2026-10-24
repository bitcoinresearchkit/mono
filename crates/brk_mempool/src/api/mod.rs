//! Read accessors on [`crate::ReadOnlyState`], each answering from one loaded publication.

use std::sync::Arc;

use brk_types::{MempoolBlockHash, Transaction};

pub mod addr;
pub mod mempool_block;
pub mod fees;
pub mod histogram;
pub mod published_tx;
pub mod rbf;
pub mod tx;

pub use addr::PendingUtxos;
pub use mempool_block::MempoolBlockSource;
pub use published_tx::PublishedTx;
pub use rbf::{RbfForTx, RbfNode};

/// A validated historical template captured for one diff request.
pub struct ResolvedMempoolBlockDiff {
    since: MempoolBlockHash,
    past: Arc<[Arc<Transaction>]>,
    source: MempoolBlockSource,
}

impl ResolvedMempoolBlockDiff {
    #[must_use]
    pub fn since(&self) -> MempoolBlockHash {
        self.since
    }

    #[must_use]
    pub fn source(&self) -> &MempoolBlockSource {
        &self.source
    }
}
