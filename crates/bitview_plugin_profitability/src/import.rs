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
        STORAGE.import(context, 100_000, |db, version| {
            let metrics = Metrics::import(&db, version, mappings, prices)?;
            Ok(Self {
                db,
                live: None,
                metrics,
            })
        })
    }
}
