use bitview_cohort::UTXOAggregate;
use bitview_vecs::{IndexSources, LazyPriceWithRatioPerBlock, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{Database, ReadableBoxedVec};

use super::{Metrics, cost_basis::CostBasisMetrics, density::DensityMetrics};

impl Metrics {
    pub fn forced_import(
        db: &Database,
        owner: &str,
        version: Version,
        indexes: &IndexSources,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let name = format!("{owner}_urpd");
        let capitalized_price_stored = UTXOAggregate::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{name}_{}_capitalized_price_cents", id.cohort_name().id),
                version,
            )
        })?;
        let capitalized_price = UTXOAggregate::from_fn(|id| {
            LazyPriceWithRatioPerBlock::from_height_source(
                &format!("{name}_{}_capitalized_price", id.cohort_name().id),
                version,
                id.select(&capitalized_price_stored),
                indexes,
                spot,
            )
        });
        Ok(Self {
            replay: Default::default(),
            buffer: Default::default(),
            cost_basis: UTXOAggregate::try_from_fn(|id| {
                CostBasisMetrics::forced_import(
                    db,
                    &id.metric_name(&format!("{owner}_cost_basis")),
                    version,
                    indexes,
                )
            })?,
            capitalized_price,
            capitalized_price_stored,
            supply_density: DensityMetrics::forced_import(db, &name, version, indexes)?,
        })
    }
}
