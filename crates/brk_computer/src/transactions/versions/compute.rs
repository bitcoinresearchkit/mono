use brk_error::Result;
use brk_indexer::Indexer;
use vecdb::Exit;

use super::{Vecs, VersionId};

impl Vecs {
    pub(crate) fn compute(&mut self, indexer: &Indexer, exit: &Exit) -> Result<()> {
        let lengths = indexer.safe_lengths();
        let starting_height = lengths.height;
        let counts = &indexer.vecs().transaction_features.count;
        self.compute_columns(
            starting_height,
            |version| match version {
                VersionId::V1 => &counts.v1,
                VersionId::V2 => &counts.v2,
                VersionId::V3 => &counts.v3,
                VersionId::Other => &counts.other_version,
            },
            exit,
        )
    }
}
