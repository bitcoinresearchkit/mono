use bitview_compute::{ColumnarDailyMetric, DailyMappings, LazyColumnDailyMetric};
use bitview_plugin::ImportContext;
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use crate::{LossPercentileId, ModeVecs, Modes, PriceBandId, STORAGE, price::LazyColumnPrice};

impl ModeVecs {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let loss_threshold = ColumnarDailyMetric::forced_import(
            db,
            &format!("{name}_loss_thresholds"),
            version,
            |source| {
                LossPercentileId::series(|percentile| {
                    LazyColumnDailyMetric::new(
                        &format!("{name}_loss_threshold_{}", percentile.suffix()),
                        version,
                        source,
                        percentile,
                        mappings,
                    )
                })
            },
        )?;

        let prices = ColumnarDailyMetric::forced_import(
            db,
            &format!("{name}_price_bands"),
            version,
            |source| {
                PriceBandId::series(|band| {
                    LazyColumnPrice::new(
                        &format!("{name}_{}", band.suffix()),
                        version,
                        source,
                        band,
                        mappings,
                    )
                })
            },
        )?;

        Ok(Self {
            loss_threshold,
            prices,
        })
    }
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let states_path = STORAGE.path(context).join("states");
        let version = STORAGE.schema_version();
        let mappings = DailyMappings::new(mappings);

        let modes = Modes::try_from_fn(|mode| {
            let name = mode.name();
            ModeVecs::forced_import(&db, &format!("bedrock_{name}"), version, &mappings)
        })?;
        let this = Self {
            plugin_gate: Default::default(),
            db,
            states_path,
            modes,
        };
        STORAGE.finalize_database(&this.db, &this)?;
        Ok(this)
    }
}
