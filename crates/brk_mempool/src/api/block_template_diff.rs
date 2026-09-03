use std::sync::Arc;

use brk_types::{NextBlockHash, Txid};

use super::BlockTemplateSource;

/// A validated historical template captured for one diff request.
pub struct ResolvedBlockTemplateDiff {
    since: NextBlockHash,
    past: Arc<[Txid]>,
    source: BlockTemplateSource,
}

impl ResolvedBlockTemplateDiff {
    pub(super) fn new(
        since: NextBlockHash,
        past: Arc<[Txid]>,
        source: BlockTemplateSource,
    ) -> Self {
        Self {
            since,
            past,
            source,
        }
    }

    #[must_use]
    pub fn since(&self) -> NextBlockHash {
        self.since
    }

    #[must_use]
    pub fn source(&self) -> &BlockTemplateSource {
        &self.source
    }

    pub(super) fn into_parts(self) -> (NextBlockHash, Arc<[Txid]>) {
        (self.since, self.past)
    }
}
