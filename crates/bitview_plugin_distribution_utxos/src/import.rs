use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_distribution_common::RealizedCaps;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Height, Sats};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec};

use crate::{SAVED_CHECKPOINTS, STORAGE, Vecs, metrics::CohortMetrics};
impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let caps = RealizedCaps::forced_import(&db, SAVED_CHECKPOINTS)?;
        let cohorts = CohortMetrics::forced_import(
            &db,
            STORAGE.schema_version(),
            mappings,
            windows,
            &prices.spot.cents.height.read_only_boxed_clone(),
            all_supply,
        )?;
        let this = Self {
            db,
            caps,
            cohorts,
            live: None,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
