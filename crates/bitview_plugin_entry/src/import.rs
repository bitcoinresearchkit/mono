use bitview_cohort::{ByEntry, CohortId};
use bitview_collections::Windows;
use bitview_distribution::metrics::ShareTotals;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height};
use vecdb::ReadableBoxedVec;

use crate::{STORAGE, Vecs, metrics::CohortMetrics};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &ReadableBoxedVec<Height, Cents>,
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let cohorts = ByEntry::try_from_fn(|id| {
            CohortMetrics::import(
                &db,
                CohortId::Entry(id),
                STORAGE.schema_version(),
                mappings,
                windows,
                prices,
                totals,
            )
        })?;
        STORAGE.finalize_database(&db)?;
        Ok(Self {
            db,
            live: None,
            cohorts,
        })
    }
}
