use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::{Inputs, write_chunks};
use bitview_distribution::state::cost_basis::{DensityBand, PRICE_INDEX_VERSION, age_index};
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_traversable::Traversable;
use bitview_vecs::Density;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
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
        let exit = context.exit();
        let sources = Sources::new(deps.age);
        let spot = &deps.price.spot.cents.height;
        let monotonic = &deps.mappings.timestamp.monotonic;
        let mut inputs = Inputs::new(
            Version::ONE + PRICE_INDEX_VERSION + age_index::history_version(deps.history),
            [spot as &dyn AnyVec, monotonic]
                .into_iter()
                .chain(sources.vectors()),
        );
        inputs.len = inputs.len.min(deps.history.len());
        let version = inputs.version;
        let range = inputs.prepare(self.state_vecs_mut(), usize::from(deps.from), exit)?;
        let (start, end) = (range.start, range.end);
        if end == 0 {
            return Ok(());
        }
        let (mut live, spots) =
            LiveState::resume(live, version, deps.history, start, end, spot, monotonic)?;
        let mut cursor = deps.history.cursor(&mut live.origins)?;
        for chunk in write_chunks(range) {
            let rows = sources.collect(chunk.start, chunk.end)?;
            for (offset, row) in rows.iter().enumerate() {
                let h = chunk.start + offset;
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
                // Without a positive spot there is no band, as in the URPD densities.
                let split = DensityBand::new(spot).map(|band| live.index.density_split(band));
                let mut percentiles = live
                    .index
                    .percentiles::<{ AgeAggregateId::ALL.len() }>(|q, n| {
                        age_index::selected(AgeAggregateId::ALL[q], n)
                    })
                    .into_iter();
                self.cost_basis.push(AgeAggregate::from_fn(|id| {
                    let (sats, cap) = age_index::selected(id, &total);
                    // Canonical balances and nested cuts keep every total nonnegative.
                    let positive = |value: i128| {
                        debug_assert!(value >= 0, "negative cost-basis total {value}");
                        value as u128
                    };
                    let (supply_density, capital_density) = if let Some((profit, loss)) = &split {
                        let (profit_sats, profit_cap) = age_index::selected(id, profit);
                        let (loss_sats, loss_cap) = age_index::selected(id, loss);
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
            self.save(chunk.end, false, exit)?;
        }
        drop(cursor);
        let all_capital = self.cohorts.all.capital_cents().read_only_boxed_clone();
        for metrics in self.cohorts.iter_mut() {
            metrics.compute_rest(
                Height::from(start),
                &self.all_supply,
                &all_capital,
                spot,
                exit,
            )?;
        }
        self.save(end, true, exit)?;
        self.live = Some(live);
        Ok(())
    }
}
impl Vecs {
    /// The block loop's outputs: the cost-basis sources and each cohort's columns. The rest
    /// derive from them after the loop, each validating its own version.
    fn state_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.cost_basis
            .iter_any_stored_mut()
            .chain(
                self.cohorts
                    .iter_mut()
                    .flat_map(|metrics| metrics.state_vecs_mut()),
            )
            .collect()
    }

    /// Saves the block loop's outputs, or with `final_chunk` every output, stamped through `end`.
    fn save(&mut self, end: usize, final_chunk: bool, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        let stamp = Stamp::from(Height::from(end - 1));
        let vectors = if final_chunk {
            self.iter_any_stored_mut().collect()
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
