use bitview_cohort::{ByEntry, CohortId};
use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{STORAGE, live::LiveState, metrics::CohortMetrics};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) db: Database,
    pub(crate) live: M::WriteOnly<Option<LiveState>>,
    pub cohorts: ByEntry<CohortMetrics<M>>,
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
        prices: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
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
                all_supply,
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
