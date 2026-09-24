use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, DailyMappings, LazyDailyPrice, import_cached};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Cents, Date, Day1, Version};
use vecdb::{AnyStoredVec, Database, ReadableVec, Rw, StorageMode, WritableVec};

use super::PriceBounds;
use crate::{
    distribution::AgeCutoffs,
    metrics::{WRITE_INTERVAL_DAYS, prepare::prepare},
};

#[derive(Traversable)]
pub struct AgeBoundsMetrics<M: StorageMode = Rw> {
    /// Daily bounds of occupied, unweighted URPD price buckets. Empty cohorts
    /// and missing snapshots are undefined. The current day uses live state
    /// with the same price-bucket rounding as saved snapshots.
    #[traversable(flatten)]
    pub series: AgeCutoffs<PriceBounds<LazyDailyPrice>>,
    #[traversable(hidden)]
    pub stored: AgeCutoffs<PriceBounds<CachedSeries<Day1, Cents, M>>>,
}

impl AgeBoundsMetrics {
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
                    &format!("utxos_urpd_{age}_cost_basis_{side}_cents"),
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

    pub fn push(&mut self, values: &AgeCutoffs<PriceBounds<Cents>>) {
        for (target, value) in self.stored.iter_mut().zip(values.iter()) {
            target.min.push(value.min);
            target.max.push(value.max);
        }
    }

    pub fn compute(
        &mut self,
        version: Version,
        from: usize,
        dates: &impl ReadableVec<Day1, Date>,
        mut load: impl FnMut(Day1, Date) -> Result<Option<AgeCutoffs<PriceBounds<Cents>>>>,
        exit: &Exit,
    ) -> Result<()> {
        let end = dates.len();
        let start = prepare(self.stored_vecs_mut(), version + dates.version(), from, end)?;
        for index in start..end {
            let day = Day1::from(index);
            let bounds = dates
                .collect_one(day)
                .map(|date| load(day, date))
                .transpose()?
                .flatten()
                .unwrap_or_default();
            self.push(&bounds);
            if (index + 1).is_multiple_of(WRITE_INTERVAL_DAYS) || index + 1 == end {
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
