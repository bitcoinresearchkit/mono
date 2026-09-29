use crate::groups::SizeGroups;
use crate::sources::SizeSources;
use bitview_cohort::CohortContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPriceWithRatioPerBlock;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

#[derive(Traversable)]
pub struct RealizedPriceByCohort<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub cohorts: SizeGroups<LazyPriceWithRatioPerBlock>,
    /// Reported in cents per BTC.
    #[traversable(hidden)]
    pub stored: SizeSources<Cents, M>,
}

impl RealizedPriceByCohort {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored = SizeSources::forced_import(db, "realized_price_cents", version)?;
        let cohorts = SizeGroups::new(|cohort_id| {
            let name = CohortContext::Utxo.metric_name(cohort_id, "realized_price");
            LazyPriceWithRatioPerBlock::from_height_source(
                &name,
                version,
                stored.get(cohort_id).expect("realized-price cohort source"),
                mappings,
                spot_price,
            )
        });
        Ok(Self { cohorts, stored })
    }
}
