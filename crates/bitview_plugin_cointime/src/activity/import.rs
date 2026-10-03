use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::{BoundedOddsF64, BoundedToF64};
use bitview_vecs::{LazyPerBlock, LazyWindowStartVec, PerBlock, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{DerivedVecs, Vecs};

impl DerivedVecs {
    fn import_with_prefix(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let name = |metric: &str| {
            if prefix.is_empty() {
                metric.to_owned()
            } else {
                format!("{prefix}_{metric}")
            }
        };
        let liveliness_name = name("liveliness");
        let version = version + Version::ONE;
        let liveliness_source =
            PerBlock::import(db, &name("liveliness_bounded_source"), version, mappings)?;
        let liveliness = LazyPerBlock::from_resolutions::<BoundedToF64>(
            &liveliness_name,
            version,
            &liveliness_source,
        );
        let vaultedness = LazyPerBlock::from_resolutions::<BoundedToF64<true>>(
            &name("vaultedness"),
            version,
            &liveliness_source,
        );
        let ratio = LazyPerBlock::from_resolutions::<BoundedOddsF64>(
            &name("activity_to_vaultedness"),
            version + Version::ONE,
            &liveliness_source,
        );

        Ok(Self {
            liveliness_source,
            liveliness,
            vaultedness,
            ratio,
        })
    }
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Vecs {
            coinblocks_created: PerBlockCumulativeRolling::import(
                db,
                "coinblocks_created",
                version,
                mappings,
                window_starts,
            )?,
            coinblocks_stored: PerBlockCumulativeRolling::import(
                db,
                "coinblocks_stored",
                version,
                mappings,
                window_starts,
            )?,
            derived: DerivedVecs::import_with_prefix(db, "", version, mappings)?,
        })
    }
}
