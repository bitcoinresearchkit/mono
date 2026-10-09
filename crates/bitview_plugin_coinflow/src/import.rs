use bitview_plugin::ImportContext;
use bitview_plugin_age::Vecs as AgeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_urpd::Metrics as UrpdMetrics;
use brk_error::Result;
use brk_types::Version;
use vecdb::ReadableCloneableVec;

use crate::{AgeRangeVecs, AggregateVecs, STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        prices: &PriceVecs,
        age: &AgeVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 250_000)?;
        let version = STORAGE.schema_version() + Version::ONE;
        let spot_price = prices.spot.cents.height.read_only_boxed_clone();

        let age_ranges = AgeRangeVecs::import(&db, version, mappings, &spot_price, age)?;
        let aggregate = AggregateVecs::import(&db, version, mappings, &spot_price)?;
        let urpd = UrpdMetrics::import(&db, "coinflow", version, mappings)?;
        let this = Self {
            db,
            age_ranges,
            aggregate,
            urpd,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
