use bitview_cohort::AgeAggregate;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPercentPerBlock, import_cached};
use brk_error::Result;
use brk_types::{Height, PartsPerMillion32, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};
#[derive(Traversable)]
pub(crate) struct DensitySources<M: StorageMode = Rw> {
    pub series: AgeAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(hidden)]
    stored: AgeAggregate<CachedSeries<Height, PartsPerMillion32, M>>,
}
impl DensitySources {
    pub fn import(db: &Database, metric: &str, v: Version, mappings: &Mappings) -> Result<Self> {
        let stored = AgeAggregate::try_from_fn(|id| {
            import_cached(db, &id.metric_name(&format!("{metric}_ppm")), v)
        })?;
        let series = AgeAggregate::from_fn(|id| {
            LazyPercentPerBlock::from_height_source(
                &id.metric_name(metric),
                v,
                id.select(&stored),
                mappings,
            )
        });
        Ok(Self { series, stored })
    }
    pub fn push(&mut self, values: AgeAggregate<PartsPerMillion32>) {
        for (v, &value) in self.stored.iter_mut().zip(values.iter()) {
            v.push(value);
        }
    }
    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}
