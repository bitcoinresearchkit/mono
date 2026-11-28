use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{IndexSources, LazyPerBlock, Price, import_cached, ops::BoundedToPercent};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ImportableVec, PcoVec, ReadableBoxedVec, ReadableCloneableVec};

use crate::{
    EntryVecs, ModeVecs, Modes, Percentiles, PriceBands, ProfitPercentiles, STORAGE, Vecs,
};

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
                mappings,
            )
        })?;
        let supply_in_loss_share_threshold = Percentiles::from_fn(|id| {
            LazyPerBlock::from_height_source::<BoundedToPercent>(
                &format!("{name}_supply_in_loss_share_threshold_{}", id.name()),
                version,
                supply_in_loss_share_threshold_stored.select(id),
                mappings,
            )
        });
        let supply_in_profit_share_threshold = ProfitPercentiles::from_fn(|id| {
            LazyPerBlock::from_height_source::<BoundedToPercent<true>>(
                &format!("{name}_supply_in_profit_share_threshold_{}", id.name()),
                version,
                supply_in_loss_share_threshold_stored.select(id.loss()),
                mappings,
            )
        });
        let prices_stored = PriceBands::try_from_fn(|id| {
            import_cached(db, &format!("{name}_{}_cents", id.suffix()), version, mappings)
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
            supply_in_profit_share_rank: PcoVec::import(
                db,
                &format!("{name}_supply_in_profit_share_rank"),
                version,
            )?,
            supply_in_profit_share_threshold,
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
            day_of_height: mappings.height.day1.read_only_boxed_clone(),
            first_height: mappings.day1.first_height.read_only_boxed_clone(),
            calibration: None,
            walk: Default::default(),
            entry_supply: Default::default(),
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
