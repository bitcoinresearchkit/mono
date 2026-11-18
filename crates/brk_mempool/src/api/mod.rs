//! Read accessors on [`crate::ReadOnlyState`], each answering from one loaded publication.

use brk_types::MempoolBlockHash;

use crate::snapshot::BlockContent;

pub mod addr;
pub mod block_template;
pub mod fees;
pub mod histogram;
pub mod mempool_block;
pub mod pending;
pub mod published_tx;
pub mod rbf;
pub mod tx;

pub use addr::{PendingUtxos, Spends};
pub use mempool_block::MempoolBlockSource;
pub use pending::PendingView;
pub use published_tx::{PublishedEntry, PublishedTx};
pub use rbf::{RbfForTx, RbfNode};

/// A validated past block captured for one diff request.
pub struct ResolvedMempoolBlockDiff {
    past: Option<BlockContent>,
    source: MempoolBlockSource,
}

impl ResolvedMempoolBlockDiff {
    #[must_use]
    pub fn since(&self) -> MempoolBlockHash {
        self.past.as_ref().map_or(MempoolBlockHash::ZERO, |past| past.hash)
    }

    #[must_use]
    pub fn source(&self) -> &MempoolBlockSource {
        &self.source
    }
}
