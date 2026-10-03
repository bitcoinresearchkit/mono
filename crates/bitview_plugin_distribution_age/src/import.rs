use bitview_cohort::{AgeRange, CohortContext};
use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_urpd::AgeBoundsMetrics;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::{Height, Sats, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec};

use crate::{STORAGE, Vecs, metrics::CohortMetrics};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();
        let spot = prices.spot.cents.height.read_only_boxed_clone();
        let cohorts =
            CohortMetrics::forced_import(&db, version, mappings, windows, &spot, all_supply)?;
        let age_bounds = AgeBoundsMetrics::forced_import(&db, version, mappings)?;
        let coindays_created = AgeRange::try_from_fn(|id| {
            PerBlockCumulativeRolling::forced_import(
                &db,
                &format!(
                    "{}_coindays_created",
                    CohortContext::Utxo.full_name(id.cohort())
                ),
                version + Version::TWO,
                mappings,
                windows,
            )
        })?;
        let coinblocks_destroyed = PerBlockCumulativeRolling::forced_import(
            &db,
            "coinblocks_destroyed",
            version + Version::TWO,
            mappings,
            windows,
        )?;
        STORAGE.finalize_database(&db)?;
        Ok(Self {
            db,
            live: None,
            cohorts,
            age_bounds,
            coindays_created,
            coinblocks_destroyed,
        })
    }
}
