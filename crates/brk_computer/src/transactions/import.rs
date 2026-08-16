use std::path::Path;

use brk_error::Result;
use brk_indexer::Indexer;
use brk_types::Version;

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, Windows,
        db_utils::{finalize_db, open_db},
    },
};

use super::{
    CountVecs, FeaturesVecs, FeesVecs, PatternsVecs, PolicyVecs, SigopsVecs, SizeVecs, Vecs,
    VersionsVecs, VolumeVecs,
};

impl Vecs {
    pub(crate) fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexer: &Indexer,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 10_000_000)?;
        let version = parent_version;

        let count = CountVecs::forced_import(&db, version, indexes, cached_starts)?;
        let features = FeaturesVecs::forced_import(&db, version, indexes, cached_starts)?;
        let size = SizeVecs::forced_import(&db, version, indexer, indexes)?;
        let fees = FeesVecs::forced_import(&db, version, indexes, cached_starts)?;
        let patterns = PatternsVecs::forced_import(&db, version, indexes, cached_starts)?;
        let policy = PolicyVecs::forced_import(&db, version, indexes, cached_starts)?;
        let sigops = SigopsVecs::forced_import(&db, version, indexes, cached_starts)?;
        let versions = VersionsVecs::forced_import(&db, version, indexes, cached_starts)?;
        let volume = VolumeVecs::forced_import(
            &db,
            version,
            indexes,
            cached_starts,
            &count.total.rolling.sum,
        )?;

        let this = Self {
            plugin_gate: Default::default(),
            db,
            count,
            features,
            size,
            fees,
            patterns,
            policy,
            sigops,
            versions,
            volume,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}
