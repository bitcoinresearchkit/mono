use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::BoundedToF64;
use bitview_vecs::{DailyMappings, LazyDailyMetric, LazyDailyPrice, import_cached};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use crate::{ModeVecs, Modes, Percentiles, PriceBands, STORAGE};

impl ModeVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let version = version + Version::TWO;
        let supply_in_loss_threshold_stored = Percentiles::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{name}_supply_in_loss_threshold_{}_bounded", id.suffix()),
                version,
            )
        })?;
        let supply_in_loss_threshold = Percentiles::from_fn(|id| {
            LazyDailyMetric::from_source::<BoundedToF64>(
                &format!("{name}_supply_in_loss_threshold_{}_ratio", id.suffix()),
                version,
                id.select(&supply_in_loss_threshold_stored),
                mappings,
            )
        });
        let prices_stored = PriceBands::try_from_fn(|id| {
            import_cached(db, &format!("{name}_{}_cents", id.suffix()), version)
        })?;
        let prices = PriceBands::from_fn(|id| {
            LazyDailyPrice::from_day1_source(
                &format!("{name}_{}", id.suffix()),
                version,
                id.select(&prices_stored),
                mappings,
            )
        });
        Ok(Self {
            supply_in_loss_threshold,
            prices,
            supply_in_loss_threshold_stored,
            prices_stored,
        })
    }
}

impl Vecs {
    pub fn import(context: ImportContext<'_>, mappings: &MappingsVecs) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let indexes = mappings;
        let mappings = DailyMappings::new(indexes);

        let modes = Modes::try_from_fn(|mode| {
            let name = mode.name();
            ModeVecs::forced_import(&db, &format!("bedrock_{name}"), version, &mappings)
        })?;
        let this = Self { db, modes };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
