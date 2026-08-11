use std::path::PathBuf;

use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{LossPercentileId, ModeVecs, Modes, PriceBandId, Vecs, price::LazyColumnPrice};
use crate::{
    indexes,
    internal::{ColumnarDailyMetric, DailyMappings, LazyColumnDailyMetric},
};

const VERSION: Version = Version::new(5);

impl ModeVecs {
    fn forced_import(
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
    pub(crate) fn forced_import(
        db: &Database,
        parent_version: Version,
        indexes: &indexes::Vecs,
        states_path: PathBuf,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        let mappings = DailyMappings::new(indexes);

        Ok(Self {
            states_path,
            modes: Modes::try_from_fn(|mode| {
                let name = mode.name();
                ModeVecs::forced_import(db, &format!("bedrock_{name}"), version, &mappings)
            })?,
        })
    }
}
