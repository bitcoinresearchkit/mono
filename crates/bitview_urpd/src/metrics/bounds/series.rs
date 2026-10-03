use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_primitives::CentsCompact;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyPerBlock, Price, import_cached};
use brk_error::{Error, Result};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, StorageMode, WritableVec};

use super::PriceBounds;

#[derive(Traversable)]
pub struct AgeBoundsMetrics<M: StorageMode = Rw> {
    /// Per-block bounds of occupied, unweighted URPD price buckets. Empty cohorts
    /// are undefined. Each value is computed directly from live age cohorts.
    #[traversable(flatten)]
    series: AgeAggregate<PriceBounds<Price<LazyPerBlock<Cents>>>>,
    #[traversable(hidden)]
    stored: AgeAggregate<PriceBounds<CachedSeries<Height, Cents, M>>>,
}

impl AgeBoundsMetrics {
    pub fn import(db: &Database, version: Version, mappings: &IndexSources) -> Result<Self> {
        let version = version + Version::new(3);
        let stored = AgeAggregate::try_from_fn(|id| {
            let age = id.name();
            let import = |side| {
                import_cached(
                    db,
                    &format!("utxos_urpd_{age}_cost_basis_{side}_cents"),
                    version,
                )
            };
            Ok::<_, Error>(PriceBounds {
                min: import("min")?,
                max: import("max")?,
            })
        })?;
        let view = |age, bounds: &PriceBounds<CachedSeries<Height, Cents>>| {
            let build = |side, source: &CachedSeries<Height, Cents>| {
                Price::from_height_source(
                    &format!("utxos_urpd_{age}_cost_basis_{side}"),
                    version,
                    source,
                    mappings,
                )
            };
            PriceBounds {
                min: build("min", &bounds.min),
                max: build("max", &bounds.max),
            }
        };
        let series = AgeAggregate::from_fn(|id| view(id.name(), id.select(&stored)));
        Ok(Self { series, stored })
    }

    fn min_len(&self) -> usize {
        self.stored
            .iter()
            .flat_map(|v| [v.min.len(), v.max.len()])
            .min()
            .unwrap_or_default()
    }

    fn push(&mut self, values: &AgeAggregate<PriceBounds<Cents>>) {
        for (target, value) in self.stored.iter_mut().zip(values.iter()) {
            target.min.push(value.min);
            target.max.push(value.max);
        }
    }

    /// Replace the current block's scalar bounds and discard any reorged suffix.
    pub fn push_block(
        &mut self,
        height: Height,
        entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
    ) -> Result<()> {
        let start = self.min_len().min(usize::from(height));
        for vec in self.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }
        while self.stored.under_4m.min.len() < usize::from(height) {
            self.push(&AgeAggregate::default());
        }
        self.push(&PriceBounds::from_age_entries(entries));
        Ok(())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .flat_map(|bounds| [&mut bounds.min as &mut dyn AnyStoredVec, &mut bounds.max])
    }
}
