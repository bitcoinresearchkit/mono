use brk_error::Result;

use std::path::Path;

use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use crate::{LossPercentileId, ModeVecs, Modes, PriceBandId, price::LazyColumnPrice};
use bitview_compute::{
    ColumnarDailyMetric, DailyMappings, LazyColumnDailyMetric,
    db_utils::{finalize_db, open_db},
};

const VERSION: Version = Version::new(5);

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
    pub fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        let db = open_db(parent_path, crate::ID.as_str(), 100_000)?;
        let states_path = parent_path.join(crate::ID.as_str()).join("states");
        let version = parent_version + VERSION;
        let mappings = DailyMappings::new(indexes);

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
        finalize_db(&this.db, &this)?;
        Ok(this)
    }
}
