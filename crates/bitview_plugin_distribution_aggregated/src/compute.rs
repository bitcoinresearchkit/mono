use crate::unrealized_data::UnrealizedData;
use crate::{
    Dependencies, Vecs, cost_basis::CostBasisBlockData, live::LiveState, sources::Sources,
};
use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_plugin_distribution_common::state::cost_basis::{PriceIndex, age_index};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{CentsCompact, Height, PartsPerMillion32, Version};
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, Stamp};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;
    fn compute(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        self.db.sync_bg_tasks()?;
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
        let (mut live, reuse) = match live {
            Some(live)
                if live.origins.len() == start
                    && live.version == version
                    && deps.history.matches(&live.origins)? =>
            {
                (live, true)
            }
            _ => (
                LiveState {
                    origins: deps.history.state_at(start)?,
                    index: PriceIndex::default(),
                    prices: Vec::new(),
                    timestamps: Vec::new(),
                    crossings: [0; 3],
                    version,
                },
                false,
            ),
        };
        let new_prices = deps
            .price
            .spot
            .cents
            .height
            .collect_range_at(live.prices.len(), end);
        if new_prices.iter().any(|v| v.is_nan()) {
            return Err(Error::NotFound("invalid aggregate price history".into()));
        }
        let price_start = live.prices.len();
        live.prices
            .extend(new_prices.iter().copied().map(CentsCompact::from));
        live.timestamps.extend(
            deps.mappings
                .timestamp
                .monotonic
                .collect_range_at(live.timestamps.len(), end),
        );
        if live.prices.len() != end || live.timestamps.len() != end {
            return Err(Error::NotFound(
                "incomplete aggregate price or timestamp history".into(),
            ));
        }
        if !reuse {
            age_index::restore(
                &mut live.index,
                &live.origins,
                &live.prices,
                &live.timestamps,
            );
        }
        let mut cursor = deps.history.cursor(&mut live.origins)?;
        for from in (start..end).step_by(10_000) {
            let next = (from + 10_000).min(end);
            let rows = sources.collect(from, next)?;
            for (offset, row) in rows.iter().enumerate() {
                let h = from + offset;
                let spot = new_prices[h - price_start];
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
        context.compact_database(&self.db);
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
        self.db.flush()?;
        Ok(())
    }
}
