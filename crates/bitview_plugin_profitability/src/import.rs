use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use brk_error::Result;
use brk_types::{Cents, Height};
use vecdb::ReadableBoxedVec;

use crate::{STORAGE, Vecs, cost_basis::CostBasisVecs, metrics::Metrics};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        prices: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let metrics = Metrics::import(&db, version, mappings, prices)?;
        let cost_basis = CostBasisVecs::import(&db, version, mappings)?;
        STORAGE.finalize_database(&db)?;
        Ok(Self {
            db,
            live: None,
            metrics,
            cost_basis,
        })
    }
}
