use bitview_cohort::{ByEntry, EntryPrice};
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Age, Cents, CostBasisSnapshot, Height, Sats, SupplyState, Version};
use statedb::Amount;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, Stamp};

use crate::{
    Dependencies, Vecs,
    live::{CohortState, LiveState},
};

fn supply(amount: Amount) -> SupplyState {
    SupplyState {
        value: Sats::new(amount.sats),
        utxo_count: amount.count,
    }
}

fn classify(price: Cents, previous: Cents) -> EntryPrice {
    EntryPrice::from_is_discount(previous == Cents::ZERO || price <= previous)
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        self.db.sync_bg_tasks()?;
        let version = (
            deps.prices.version(),
            deps.timestamps.version(),
            deps.capitalized_price.version(),
            deps.history.versions(),
        );
        let base_version = Version::ONE
            + version.0
            + version.1
            + version.2
            + Version::from(version.3.0 as u32)
            + Version::from(version.3.1 as u32);
        {
            let _lock = context.exit().lock();
            for vec in self.state_vecs_mut() {
                vec.any_validate_computed_version_or_reset(base_version)?;
            }
        }
        let end = deps
            .history
            .len()
            .min(deps.prices.len())
            .min(deps.timestamps.len())
            .min(deps.capitalized_price.len());
        let start = usize::from(deps.from).min(end).min(
            self.cohorts
                .iter()
                .map(|c| c.min_len())
                .min()
                .unwrap_or_default(),
        );
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
                    cohorts: ByEntry::from_fn(|_| CohortState::new()),
                    entries: Vec::new(),
                    prices: Vec::new(),
                    timestamps: Vec::new(),
                    version,
                },
                false,
            ),
        };
        let data_start = live.prices.len();
        live.prices
            .extend(deps.prices.collect_range_at(data_start, end));
        live.timestamps
            .extend(deps.timestamps.collect_range_at(data_start, end));
        let anchor_start = data_start.saturating_sub(1);
        let anchors = deps
            .capitalized_price
            .collect_range_at(anchor_start, end.saturating_sub(1));
        if live.prices.len() != end
            || live.timestamps.len() != end
            || anchors.len() != end.saturating_sub(1) - anchor_start
        {
            return Err(Error::NotFound(
                "incomplete entry price or timestamp history".into(),
            ));
        }
        live.entries.extend((data_start..end).map(|h| {
            classify(
                live.prices[h],
                h.checked_sub(1)
                    .map_or(Cents::ZERO, |p| anchors[p - anchor_start]),
            )
        }));
        if !reuse {
            for (h, &amount) in live.origins.amounts().iter().enumerate() {
                live.cohorts.get_mut(live.entries[h]).increment_snapshot(
                    &CostBasisSnapshot::from_utxo(live.prices[h], &supply(amount)),
                );
            }
            for state in live.cohorts.iter_mut() {
                state.finish_restore();
            }
        }
        {
            let _lock = context.exit().lock();
            for vec in self.state_vecs_mut() {
                vec.any_truncate_if_needed_at(start)?;
            }
        }
        let mut cursor = deps.history.cursor(&mut live.origins)?;
        for h in start..end {
            let price = live.prices[h];
            let diff = cursor.advance()?.expect("validated entry history range");
            live.cohorts
                .get_mut(live.entries[h])
                .receive_utxo(&supply(diff.created), price);
            for (origin, amount) in diff.removed() {
                let origin = origin as usize;
                let age = Age::new(live.timestamps[h], live.timestamps[origin]);
                live.cohorts.get_mut(live.entries[origin]).send_utxo(
                    &supply(amount),
                    price,
                    live.prices[origin],
                    price,
                    age,
                );
            }
            for (target, state) in self.cohorts.iter_mut().zip(live.cohorts.iter_mut()) {
                state.apply_pending();
                target.push(state, price);
                state.reset_single_iteration_values();
            }
            if (h + 1) % 10_000 == 0 || h + 1 == end {
                self.save(h + 1, context.exit())?;
            }
        }
        for cohort in self.cohorts.iter_mut() {
            cohort.compute_rest(Height::from(start), context.exit())?;
        }
        context.compact_database(&self.db);
        self.live = Some(live);
        Ok(())
    }
}

impl Vecs {
    fn state_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.cohorts
            .iter_mut()
            .flat_map(|cohort| cohort.stored_vecs_mut())
            .collect()
    }

    fn save(&mut self, end: usize, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        let stamp = Stamp::from(Height::from(end - 1));
        for vec in self.state_vecs_mut() {
            vec.any_stamped_write_maybe_with_changes(stamp, true)?;
        }
        self.db.flush()?;
        Ok(())
    }
}
