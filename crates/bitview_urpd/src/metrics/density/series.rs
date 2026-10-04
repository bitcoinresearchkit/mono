use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyPercentPerBlock, import_cached};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use super::SupplyDensity;

#[derive(Traversable)]
pub struct DensitySeries<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub series: SupplyDensity<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(hidden)]
    pub stored: SupplyDensity<CachedSeries<Height, PartsPerMillion32, M>>,
}

impl DensitySeries {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let stored = SupplyDensity::try_from_fn(|suffix| {
            import_cached(db, &format!("{name}{suffix}_ppm"), version)
        })?;
        let view = |source: &CachedSeries<Height, PartsPerMillion32>, suffix| {
            LazyPercentPerBlock::from_height_source(
                &format!("{name}{suffix}"),
                version,
                source,
                mappings,
            )
        };
        let series = SupplyDensity {
            total: view(&stored.total, "_total"),
            in_profit: view(&stored.in_profit, "_in_profit"),
            in_loss: view(&stored.in_loss, "_in_loss"),
        };
        Ok(Self { series, stored })
    }

    pub fn push(&mut self, density: &SupplyDensity<PartsPerMillion32>) {
        for (target, &value) in self.stored.iter_mut().zip(density.iter()) {
            target.push(value);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|vec| vec as &mut dyn AnyStoredVec)
    }
}
