use brk_error::Result;

use brk_types::Version;
use vecdb::{Database, ReadableCloneableVec};

use super::{DerivedVecs, Vecs};
use bitview_compute::{
    CachedWindowStartVec, LazyPerBlock, OddsF64, OneMinusF64, PerBlock, PerBlockCumulativeRolling,
    Windows,
};

impl DerivedVecs {
    fn forced_import_with_prefix(
        db: &Database,
        prefix: &str,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        let name = |metric: &str| {
            if prefix.is_empty() {
                metric.to_owned()
            } else {
                format!("{prefix}_{metric}")
            }
        };
        let liveliness_name = name("liveliness");
        let liveliness = PerBlock::forced_import(db, &liveliness_name, version, indexes)?;
        let vaultedness = LazyPerBlock::from_computed::<OneMinusF64>(
            &name("vaultedness"),
            version,
            liveliness.height.read_only_boxed_clone(),
            &liveliness,
        );
        let ratio = LazyPerBlock::from_computed::<OddsF64>(
            &name("activity_to_vaultedness"),
            version,
            liveliness.height.read_only_boxed_clone(),
            &liveliness,
        );

        Ok(Self {
            liveliness,
            vaultedness,
            ratio,
        })
    }
}

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
    cached_starts: &Windows<&CachedWindowStartVec>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes, cached_starts)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            coinblocks_created: PerBlockCumulativeRolling::forced_import(
                db,
                "coinblocks_created",
                version,
                indexes,
                cached_starts,
            )?,
            coinblocks_stored: PerBlockCumulativeRolling::forced_import(
                db,
                "coinblocks_stored",
                version,
                indexes,
                cached_starts,
            )?,
            derived: DerivedVecs::forced_import_with_prefix(db, "", version, indexes)?,
        })
    }
}
