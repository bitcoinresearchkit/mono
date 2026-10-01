use bitview_cohort::UTXOAggregate;
use brk_error::Result;
use brk_types::Version;
use vecdb::{Database, Rw};

use crate::{AggregatePerBlock, FiatType, IndexSources, LazyFiatPerBlock, import_cached};

pub type AggregateFiatPerBlock<C, M = Rw> = AggregatePerBlock<LazyFiatPerBlock<C>, C, M>;

impl<C: FiatType> AggregateFiatPerBlock<C> {
    pub fn forced_import(
        db: &Database,
        metric: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let stored = UTXOAggregate::try_from_fn(|id| {
            import_cached(
                db,
                &format!("{}_cents", id.metric_name(metric)),
                version + Version::ONE,
            )
        })?;
        let series = UTXOAggregate::from_fn(|id| {
            LazyFiatPerBlock::from_cents_source(
                &id.metric_name(metric),
                version,
                id.select(&stored),
                indexes,
            )
        });
        Ok(Self { series, stored })
    }
}
