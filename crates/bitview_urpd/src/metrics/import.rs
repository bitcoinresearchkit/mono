use std::path::PathBuf;

use bitview_cohort::UTXOAggregate;
use bitview_vecs::{DailyMappings, IndexSources, LazyDailyPriceWithRatio, import_cached};
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
        states_path: PathBuf,
    ) -> Result<Self> {
        let mappings = DailyMappings::new(indexes);
        let name = format!("{owner}_urpd");
        let capitalized_price_stored = UTXOAggregate::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{name}_{}_capitalized_price_cents", id.cohort_name().id),
                version,
            )
        })?;
        let capitalized_price = UTXOAggregate::from_fn(|id| {
            LazyDailyPriceWithRatio::from_day1_source(
                &format!("{name}_{}_capitalized_price", id.cohort_name().id),
                version,
                id.select(&capitalized_price_stored),
                indexes,
                &mappings,
                spot,
            )
        });
        Ok(Self {
            cost_basis: UTXOAggregate::try_from_fn(|id| {
                CostBasisMetrics::forced_import(
                    db,
                    &id.metric_name(&format!("{owner}_cost_basis")),
                    version,
                    &mappings,
                )
            })?,
            capitalized_price,
            capitalized_price_stored,
            supply_density: DensityMetrics::forced_import(db, &name, version, &mappings)?,
            states_path,
        })
    }
}
