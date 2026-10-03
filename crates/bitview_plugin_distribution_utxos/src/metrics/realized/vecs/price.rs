use crate::sources::UtxoSources;
use bitview_cohort::CohortContext;
use bitview_cohort::UtxoGroups;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, Price};
use brk_error::Result;
use brk_types::{Cents, Version};
use vecdb::{Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct RealizedPriceByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: UtxoGroups<Price<LazyPerBlock<Cents>>>,
    /// Reported in cents per BTC.
    #[traversable(hidden)]
    pub stored: UtxoSources<Cents, M>,
}

impl RealizedPriceByCohort {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let version = version + Version::ONE;
        let stored = UtxoSources::forced_import(db, "realized_price_cents", version)?;
        let cohorts = UtxoGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_price");
            Price::from_height_source(
                &name,
                version,
                stored.get(cohort_id).expect("realized-price cohort source"),
                mappings,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
