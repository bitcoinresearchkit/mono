use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::prepare_computed;
use bitview_distribution::state::cost_basis::age_index;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_primitives::PartsPerMillion32;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, Stamp};

use crate::{
    Dependencies, Vecs, cost_basis::CostBasisBlockData, live::LiveState, sources::Sources,
    unrealized_data::UnrealizedData,
};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }
    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        let sources = Sources::new(deps.age);
        let version = (
            deps.price.spot.cents.height.version(),
            deps.mappings.timestamp.monotonic.version(),
            deps.history.versions(),
            sources.version(),
        );
        let base_version = Version::ONE
            + version.0
            + version.1
            + version.3
            + Version::from(version.2.0 as u32)
            + Version::from(version.2.1 as u32);
        let end = deps
            .history
            .len()
            .min(deps.price.spot.cents.height.len())
            .min(deps.mappings.timestamp.monotonic.len())
            .min(sources.len());
        let start = prepare_computed(
            self.state_vecs_mut(),
            base_version,
            usize::from(deps.from).min(end),
            context.exit(),
        )?;
        if end == 0 {
            return Ok(());
        }
        let (mut live, spots) = LiveState::resume(
            live,
            version,
            deps.history,
            start,
            end,
            &deps.price.spot.cents.height,
            &deps.mappings.timestamp.monotonic,
        )?;
        let mut cursor = deps.history.cursor(&mut live.origins)?;
        for from in (start..end).step_by(10_000) {
            let next = (from + 10_000).min(end);
            let rows = sources.collect(from, next)?;
            for (offset, row) in rows.iter().enumerate() {
                let h = from + offset;
                let spot = spots[h - start];
                age_index::advance(
                    &mut live.index,
                    &mut cursor,
                    &live.prices,
                    &live.timestamps,
                    &mut live.crossings,
                )?;
                let unrealized =
                    AgeAggregate::from_fn(|id| UnrealizedData::new(spot, id.select(row)));
                self.cost_basis.push_prices(&unrealized);
                let total = live.index.totals();
                let density = live.index.density_range(spot);
                let mut percentiles = live
                    .index
                    .percentiles::<{ AgeAggregateId::ALL.len() }>(|q, n| {
                        age_index::selected(AgeAggregateId::ALL[q], n)
                    })
                    .into_iter();
                self.cost_basis.push(AgeAggregate::from_fn(|id| {
                    let sats = age_index::selected(id, &total).0;
                    let density = if sats > 0 {
                        PartsPerMillion32::from(
                            age_index::selected(id, &density).0.max(0) as f64 / sats as f64,
                        )
                    } else {
                        PartsPerMillion32::ZERO
                    };
                    CostBasisBlockData::from_percentiles(
                        percentiles.next().expect("one result per age filter"),
                        density,
                    )
                }));
                for id in AgeAggregateId::ALL {
                    id.select_mut(&mut self.cohorts)
                        .push(id.select(row), id.select(&unrealized));
                }
            }
            self.save(next, false, context.exit())?;
        }
        drop(cursor);
        for metrics in self.cohorts.iter_mut() {
            metrics.compute_rest(
                Height::from(start),
                &self.all_supply,
                &self.all_market_cap,
                context.exit(),
            )?;
        }
        self.save(end, true, context.exit())?;
        self.live = Some(live);
        Ok(())
    }
}
impl Vecs {
    fn state_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut v = self.cost_basis.collect_vecs_mut();
        for metrics in self.cohorts.iter_mut() {
            v.extend(metrics.state_vecs_mut());
        }
        v
    }
    fn stored_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut v = self.cost_basis.collect_vecs_mut();
        for metrics in self.cohorts.iter_mut() {
            v.extend(metrics.stored_vecs_mut());
        }
        v
    }
    fn save(&mut self, end: usize, final_chunk: bool, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        let stamp = Stamp::from(Height::from(end - 1));
        let vectors = if final_chunk {
            self.stored_vecs_mut()
        } else {
            self.state_vecs_mut()
        };
        for v in vectors {
            v.any_stamped_write_maybe_with_changes(stamp, final_chunk)?;
        }
        self.db.flush();
        Ok(())
    }
}
