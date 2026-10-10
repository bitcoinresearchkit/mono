use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::prepare_computed;
use bitview_distribution::state::cost_basis::{PRICE_INDEX_VERSION, age_index};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_vecs::Density;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableCloneableVec, Stamp};

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
            + PRICE_INDEX_VERSION
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
                let total = live.index.totals();
                let (profit, loss) = live.index.density_split(spot);
                let mut percentiles = live
                    .index
                    .percentiles::<{ AgeAggregateId::ALL.len() }>(|q, n| {
                        age_index::selected(AgeAggregateId::ALL[q], n)
                    })
                    .into_iter();
                self.cost_basis.push(AgeAggregate::from_fn(|id| {
                    let (sats, cap) = age_index::selected(id, &total);
                    let (profit_sats, profit_cap) = age_index::selected(id, &profit);
                    let (loss_sats, loss_cap) = age_index::selected(id, &loss);
                    let positive = |value: i128| value.max(0) as u128;
                    // Without a positive spot there is no band, as in the URPD densities.
                    let (supply_density, capital_density) = if spot > Cents::ZERO {
                        (
                            Density::from_sums(
                                positive(sats.into()),
                                positive(profit_sats.into()),
                                positive(loss_sats.into()),
                            ),
                            Density::from_sums(
                                positive(cap),
                                positive(profit_cap),
                                positive(loss_cap),
                            ),
                        )
                    } else {
                        (Density::NAN, Density::NAN)
                    };
                    CostBasisBlockData::from_percentiles(
                        percentiles.next().expect("one result per age filter"),
                        supply_density,
                        capital_density,
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
        let all_capital = self.cohorts.all.capital_cents().read_only_boxed_clone();
        for metrics in self.cohorts.iter_mut() {
            metrics.compute_rest(
                Height::from(start),
                &self.all_supply,
                &all_capital,
                &deps.price.spot.cents.height,
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
