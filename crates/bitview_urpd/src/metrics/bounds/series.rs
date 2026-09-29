use bitview_cohort::AgeRangeId;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, IndexSources, LazyPerBlock, Price, import_cached};
use brk_error::{Error, Result};
use brk_types::{Cents, CentsCompact, Height, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, StorageMode, WritableVec};

use super::PriceBounds;
use crate::distribution::AgeCutoffs;

#[derive(Traversable)]
pub struct AgeBoundsMetrics<M: StorageMode = Rw> {
    /// Per-block bounds of occupied, unweighted URPD price buckets. Empty cohorts
    /// are undefined. Each value is computed directly from live age cohorts.
    #[traversable(flatten)]
    pub series: AgeCutoffs<PriceBounds<Price<LazyPerBlock<Cents>>>>,
    #[traversable(hidden)]
    pub stored: AgeCutoffs<PriceBounds<CachedSeries<Height, Cents, M>>>,
}

impl AgeBoundsMetrics {
    pub fn forced_import(db: &Database, version: Version, mappings: &IndexSources) -> Result<Self> {
        let version = version + Version::TWO;
        let stored = AgeCutoffs::try_from_fn(|age| {
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
        let series = AgeCutoffs {
            under_4m: view("under_4m", &stored.under_4m),
            under_5m: view("under_5m", &stored.under_5m),
            under_6m: view("under_6m", &stored.under_6m),
        };
        Ok(Self { series, stored })
    }

    pub fn len(&self) -> usize {
        self.stored
            .iter()
            .flat_map(|v| [v.min.len(), v.max.len()])
            .min()
            .unwrap_or_default()
    }

    fn push(&mut self, values: &AgeCutoffs<PriceBounds<Cents>>) {
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
        let start = self.len().min(usize::from(height));
        for vec in self.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }
        while self.stored.under_4m.min.len() < usize::from(height) {
            self.push(&AgeCutoffs::default());
        }
        self.push(&AgeCutoffs::from_age_entries(entries));
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        for vec in self.stored_vecs_mut() {
            vec.write()?;
        }
        Ok(())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .flat_map(|bounds| [&mut bounds.min as &mut dyn AnyStoredVec, &mut bounds.max])
    }
}
