use bitview_collections::Windows;
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{CoinBlocks, PartsPerMillion64};
use bitview_transforms::{BoundedOdds, BoundedToRatio, Quotient};
use bitview_vecs::{
    LazyPerBlock, LazyPerBlockCumulativeRolling, LazyRatioRollingWindows, LazyWindowStartVec,
    PerBlock, PerBlockCumulativeRolling,
};
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
        let liveliness = LazyPerBlock::from_resolutions::<BoundedToRatio>(
            &liveliness_name,
            version,
            &liveliness_source,
        );
        let vaultedness = LazyPerBlock::from_resolutions::<BoundedToRatio<true>>(
            &name("vaultedness"),
            version,
            &liveliness_source,
        );
        let liveliness_to_vaultedness = LazyPerBlock::from_resolutions::<BoundedOdds>(
            &name("liveliness_to_vaultedness"),
            version + Version::ONE,
            &liveliness_source,
        );

        Ok(Self {
            liveliness_source,
            liveliness,
            vaultedness,
            liveliness_to_vaultedness,
        })
    }
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        age: &AgeVecs,
    ) -> Result<Self> {
        let coinblocks_created: PerBlockCumulativeRolling<CoinBlocks> =
            PerBlockCumulativeRolling::import(
                db,
                "coinblocks_created",
                version,
                mappings,
                window_starts,
            )?;
        let concurrent_liveliness = LazyRatioRollingWindows::from_cumulative_ratio_with_numerator::<
            CoinBlocks,
            CoinBlocks,
            Quotient<PartsPerMillion64>,
        >(
            "concurrent_liveliness",
            version,
            age.coinblocks_destroyed.cumulative_source(),
            coinblocks_created.cumulative_source(),
            window_starts,
            mappings,
        );
        let coinblocks_destroyed = LazyPerBlockCumulativeRolling::from_cumulative_source(
            "coinblocks_destroyed",
            version,
            age.coinblocks_destroyed.cumulative_source(),
            window_starts,
            mappings,
        );
        Ok(Vecs {
            coinblocks_created,
            coinblocks_destroyed,
            coinblocks_stored: PerBlockCumulativeRolling::import(
                db,
                "coinblocks_stored",
                version,
                mappings,
                window_starts,
            )?,
            derived: DerivedVecs::import_with_prefix(db, "", version, mappings)?,
            concurrent_liveliness,
        })
    }
}
