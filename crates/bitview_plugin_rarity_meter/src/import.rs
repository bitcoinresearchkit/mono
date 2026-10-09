use bitview_plugin::ImportContext;
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;

use crate::{STORAGE, Vecs, components, extremes, inner};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        holders: &HoldersVecs,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let version = STORAGE.schema_version();
        let this = Self {
            components: components::Components::import(
                &db, version, mappings, holders, cointime, coinflow,
            )?,
            extremes: extremes::Extremes::import(&db, version, mappings)?,
            full: inner::RarityMeterInner::import(&db, "rarity_meter", version, mappings)?,
            full_v2: inner::RarityMeterInner::import(&db, "rarity_meter_v2", version, mappings)?,
            local: inner::RarityMeterInner::import(&db, "local_rarity_meter", version, mappings)?,
            local_v2: inner::RarityMeterInner::import(
                &db,
                "local_rarity_meter_v2",
                version,
                mappings,
            )?,
            cycle: inner::RarityMeterInner::import(&db, "cycle_rarity_meter", version, mappings)?,
            cycle_v2: inner::RarityMeterInner::import(
                &db,
                "cycle_rarity_meter_v2",
                version,
                mappings,
            )?,
            db,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
