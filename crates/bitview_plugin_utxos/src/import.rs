use bitview_distribution::{RealizedCaps, metrics::ShareTotals};
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use vecdb::ReadableCloneableVec;

use crate::{SAVED_CHECKPOINTS, STORAGE, Vecs, metrics::CohortMetrics};
impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &PriceVecs,
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        STORAGE.import(context, 20_000_000, |db, version| {
            let caps = RealizedCaps::import(&db, SAVED_CHECKPOINTS)?;
            let cohorts = CohortMetrics::import(
                &db,
                version,
                mappings,
                windows,
                &prices.spot.cents.height.read_only_boxed_clone(),
                totals,
            )?;
            let this = Self {
                db,
                caps,
                cohorts,
                live: None,
            };
            Ok(this)
        })
    }
}
