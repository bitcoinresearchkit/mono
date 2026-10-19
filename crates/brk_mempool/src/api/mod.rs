//! Read accessors on [`crate::ReadOnlyState`], each answering from one loaded publication.

use std::sync::Arc;

use brk_types::{NextBlockHash, Transaction};

pub mod addr;
pub mod block_template;
pub mod fees;
pub mod histogram;
pub mod published_tx;
pub mod rbf;
pub mod tx;

pub use addr::PendingUtxos;
pub use block_template::BlockTemplateSource;
pub use published_tx::PublishedTx;
pub use rbf::{RbfForTx, RbfNode};

/// A validated historical template captured for one diff request.
pub struct ResolvedBlockTemplateDiff {
    since: NextBlockHash,
    past: Arc<[Arc<Transaction>]>,
    source: BlockTemplateSource,
}

impl ResolvedBlockTemplateDiff {
    #[must_use]
    pub fn since(&self) -> NextBlockHash {
        self.since
    }

    #[must_use]
    pub fn source(&self) -> &BlockTemplateSource {
        &self.source
    }
}
