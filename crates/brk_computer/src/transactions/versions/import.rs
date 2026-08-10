use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, ReadOnlyClone};

use super::{Vecs, VersionId};
use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling,
        LazyColumnPerBlockCumulativeRolling, Windows,
    },
};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let source = ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            "tx_version_count_cumulative",
            version,
            |_| (),
        )?;
        let counts = source.cumulative.read_only_clone();
        let import = |name, version_id| {
            LazyColumnPerBlockCumulativeRolling::new(
                name,
                version,
                &counts,
                version_id,
                indexes,
                cached_starts,
            )
        };

        Ok(Self {
            v1: import("tx_v1", VersionId::V1),
            v2: import("tx_v2", VersionId::V2),
            v3: import("tx_v3", VersionId::V3),
            other: import("tx_other_version", VersionId::Other),
            source,
        })
    }
}
