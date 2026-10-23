use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{IndexSources, LazyPerBlock, Price, import_cached, ops::FixedToPercent};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use crate::{EntryVecs, ModeVecs, Modes, Percentiles, PriceBands, STORAGE, Vecs};

impl ModeVecs {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::new(4);
        let supply_in_loss_share_threshold_stored = Percentiles::try_from_fn(|id| {
            import_cached(
                db,
                &format!(
                    "{name}_supply_in_loss_share_threshold_{}_bounded",
                    id.name()
                ),
                version,
            )
        })?;
        let supply_in_loss_share_threshold = Percentiles::from_fn(|id| {
            LazyPerBlock::from_height_source::<FixedToPercent>(
                &format!("{name}_supply_in_loss_share_threshold_{}", id.name()),
                version,
                supply_in_loss_share_threshold_stored.select(id),
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
                prices_stored.select(id),
                mappings,
            )
        });
        let floor = EntryVecs::import(db, name, version, mappings, spot_price)?;
        Ok(Self {
            supply_in_loss_share_threshold,
            prices,
            floor,
            supply_in_loss_share_threshold_stored,
            prices_stored,
        })
    }
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();

        let modes = Modes::try_from_fn(|mode| {
            let name = mode.name();
            ModeVecs::import(
                &db,
                &format!("bedrock_{name}"),
                version,
                mappings,
                spot_price,
            )
        })?;
        let this = Self {
            db,
            modes,
            calibration: None,
            replay: Default::default(),
            entry_supply: Default::default(),
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
