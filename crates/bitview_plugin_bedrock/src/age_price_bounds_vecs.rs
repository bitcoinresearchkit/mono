use std::path::Path;

use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, DailyMappings, LazyDailyPrice, import_cached};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Cents, Date, Day1, Version};
use vecdb::{AnyStoredVec, Database, ReadableVec, Rw, StorageMode, WritableVec};

use crate::{AgeCutoffs, PriceBounds, WRITE_INTERVAL_DAYS};

#[derive(Traversable)]
pub struct AgePriceBoundsVecs<M: StorageMode = Rw> {
    /// Daily bounds of occupied, unweighted URPD price buckets. Empty cohorts
    /// and missing snapshots are undefined. The current day uses live state
    /// with the same price-bucket rounding as saved snapshots.
    #[traversable(flatten)]
    pub series: AgeCutoffs<PriceBounds<LazyDailyPrice>>,
    #[traversable(hidden)]
    pub stored: AgeCutoffs<PriceBounds<CachedSeries<Day1, Cents, M>>>,
}

impl AgePriceBoundsVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let stored = AgeCutoffs::try_from_fn(|age| {
            let import = |side| {
                import_cached(
                    db,
                    &format!("bedrock_{age}_cost_basis_{side}_cents"),
                    version,
                )
            };
            Ok::<_, Error>(PriceBounds {
                min: import("min")?,
                max: import("max")?,
            })
        })?;
        let view = |age, bounds: &PriceBounds<CachedSeries<Day1, Cents>>| {
            let build = |side, source: &CachedSeries<Day1, Cents>| {
                LazyDailyPrice::from_day1_source(
                    &format!("bedrock_{age}_cost_basis_{side}"),
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

    pub fn prepare(
        &mut self,
        version: Version,
        recompute_from: usize,
        end: usize,
    ) -> Result<usize> {
        let mut start = recompute_from.min(end);
        for vec in self.stored_vecs_mut() {
            vec.any_validate_computed_version_or_reset(version)?;
            start = start.min(vec.len());
        }
        for vec in self.stored_vecs_mut() {
            vec.any_truncate_if_needed_at(start)?;
        }
        Ok(start)
    }

    pub fn push(&mut self, values: &AgeCutoffs<PriceBounds<Cents>>) {
        for (target, value) in self.stored.iter_mut().zip(values.iter()) {
            target.min.push(value.min);
            target.max.push(value.max);
        }
    }

    pub fn backfill(
        &mut self,
        dates: &impl ReadableVec<Day1, Date>,
        states_path: &Path,
        start: usize,
        end: usize,
        exit: &Exit,
    ) -> Result<()> {
        for day in start..end {
            let bounds = match dates.collect_one(Day1::from(day)) {
                Some(date) => AgeCutoffs::read_if_exists(states_path, date)?,
                None => AgeCutoffs::default(),
            };
            self.push(&bounds);
            if (day + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || day + 1 == end {
                let _lock = exit.lock();
                for vec in self.stored_vecs_mut() {
                    vec.write()?;
                }
            }
        }
        Ok(())
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .flat_map(|bounds| [&mut bounds.min as &mut dyn AnyStoredVec, &mut bounds.max])
    }
}
