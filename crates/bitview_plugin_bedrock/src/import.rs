use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::BoundedToRatio;
use bitview_vecs::{IndexSources, LazyPerBlock, Price, import_cached};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use crate::{ModeVecs, Modes, Percentiles, PriceBands, STORAGE, Vecs};

impl ModeVecs {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let version = version + Version::new(3);
        let supply_in_loss_threshold_stored = Percentiles::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{name}_supply_in_loss_threshold_{}_bounded", id.suffix()),
                version,
            )
        })?;
        let supply_in_loss_threshold = Percentiles::from_fn(|id| {
            LazyPerBlock::from_height_source::<BoundedToRatio>(
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
            Price::from_height_source(
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

        let modes = Modes::try_from_fn(|mode| {
            let name = mode.name();
            ModeVecs::import(&db, &format!("bedrock_{name}"), version, mappings)
        })?;
        let this = Self {
            db,
            modes,
            calibration: None,
            replay: Default::default(),
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
