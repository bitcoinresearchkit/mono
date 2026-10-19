use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use brk_error::Result;
use brk_types::{Cents, Height};
use vecdb::ReadableBoxedVec;

use crate::{STORAGE, Vecs, metrics::Metrics};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        prices: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let metrics = Metrics::import(&db, STORAGE.schema_version(), mappings, prices)?;
        STORAGE.finalize_database(&db)?;
        Ok(Self {
            db,
            live: None,
            metrics,
        })
    }
}
