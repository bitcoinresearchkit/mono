use bitview_collections::Percent;
use bitview_transforms::{FixedToPercent, FixedToRatio};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyPerBlock, import_cached};
use brk_error::Result;
use brk_types::{Height, PartsPerMillion32, StoredF32, Version};
use vecdb::{AnyStoredVec, Database, Ident, Rw, StorageMode, WritableVec};

use super::SupplyDensity;

#[derive(Traversable)]
pub struct DensitySeries<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub series: SupplyDensity<
        Percent<
            LazyPerBlock<PartsPerMillion32, PartsPerMillion32>,
            LazyPerBlock<StoredF32, PartsPerMillion32>,
        >,
    >,
    #[traversable(hidden)]
    pub stored: SupplyDensity<CachedSeries<Height, PartsPerMillion32, M>>,
}

impl DensitySeries {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let stored = SupplyDensity::try_from_fn(|suffix| {
            import_cached(db, &format!("{name}{suffix}_ppm"), version)
        })?;
        let view = |source: &CachedSeries<Height, PartsPerMillion32>, suffix| {
            let name = format!("{name}{suffix}");
            Percent {
                ppm: LazyPerBlock::from_height_source::<Ident>(
                    &format!("{name}_ppm"),
                    version,
                    source,
                    mappings,
                ),
                ratio: LazyPerBlock::from_height_source::<FixedToRatio>(
                    &format!("{name}_ratio"),
                    version,
                    source,
                    mappings,
                ),
                percent: LazyPerBlock::from_height_source::<FixedToPercent>(
                    &name, version, source, mappings,
                ),
            }
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
