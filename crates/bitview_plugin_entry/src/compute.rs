use bitview_cohort::{Age, ByEntry, EntryPrice};
use bitview_compute::{Inputs, save_stamped, write_chunks};
use bitview_distribution::state::cost_basis::age_index;
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::{CostBasisSnapshot, SupplyState};
use bitview_traversable::Traversable;
use brk_error::{Error, Result};
use brk_types::{Cents, Sats, Version};
use statedb::Amount;
use vecdb::{AnyVec, ReadableVec};

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

/// Veteran at or below the previous block's capitalized price, and while it is undefined (no
/// priced capital yet).
fn classify(price: Cents, previous: Cents) -> EntryPrice {
    EntryPrice::from_is_discount(previous.is_nan() || price <= previous)
}

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        let Dependencies {
            history,
            from,
            prices,
            timestamps,
            capitalized_price,
        } = deps;
        let mut inputs = Inputs::new(
            Version::ONE + age_index::history_version(history),
            [
                prices as &dyn AnyVec,
                timestamps as &dyn AnyVec,
                capitalized_price as &dyn AnyVec,
            ],
        );
        inputs.len = inputs.len.min(history.len());
        let version = inputs.version;
        let range = inputs.prepare(
            self.cohorts.iter_any_stored_mut().collect::<Vec<_>>(),
            usize::from(from),
            context.exit(),
        )?;
        let (start, end) = (range.start, range.end);
        if end == 0 {
            return Ok(());
        }
        let (mut live, reuse) = match live {
            Some(live) if live.version == version && history.resumes(&live.origins, start)? => {
                (live, true)
            }
            _ => (
                LiveState {
                    origins: history.state_at(start)?,
                    cohorts: ByEntry::from_fn(|_| CohortState::default()),
                    entries: Vec::new(),
                    prices: Vec::new(),
                    timestamps: Vec::new(),
                    version,
                },
                false,
            ),
        };
        let data_start = live.prices.len();
        prices.read_into_at(data_start, end, &mut live.prices);
        timestamps.read_into_at(data_start, end, &mut live.timestamps);
        let anchor_start = data_start.saturating_sub(1);
        let anchors = capitalized_price.collect_range_at(anchor_start, end.saturating_sub(1));
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
                    .map_or(Cents::NAN, |p| anchors[p - anchor_start]),
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
        let mut cursor = history.cursor(&mut live.origins)?;
        for chunk in write_chunks(range) {
            for h in chunk.clone() {
                let price = live.prices[h];
                let diff = cursor.advance()?.expect("validated entry history range");
                live.cohorts
                    .get_mut(live.entries[h])
                    .receive_utxo(&supply(diff.created), price);
                for (origin, amount) in diff.spent() {
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
                // A BIP30-overwritten coinbase leaves the set without being spent.
                if let Some((origin, amount)) = diff.correction() {
                    let origin = origin as usize;
                    live.cohorts
                        .get_mut(live.entries[origin])
                        .decrement_snapshot(&CostBasisSnapshot::from_utxo(
                            live.prices[origin],
                            &supply(amount),
                        ));
                }
                for (target, state) in self.cohorts.iter_mut().zip(live.cohorts.iter_mut()) {
                    target.push(state, price);
                    state.reset_single_iteration_values();
                }
            }
            save_stamped(
                &self.db,
                self.cohorts.iter_any_stored_mut(),
                chunk.end,
                true,
                context.exit(),
            )?;
        }
        self.live = Some(live);
        Ok(())
    }
}
