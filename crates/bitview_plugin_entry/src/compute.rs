use bitview_cohort::{Age, ByEntry, EntryPrice};
use bitview_compute::{save_stamped, write_chunks};
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

/// Discount at or below the previous block's capitalized price, and while it is undefined (no
/// priced capital yet).
fn classify(price: Cents, previous: Cents) -> EntryPrice {
    EntryPrice::from_is_discount(previous.is_nan() || price <= previous)
}

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let live = self.live.take();
        let Dependencies {
            sources,
            walk,
            from,
            capitalized_price,
        } = deps;
        let history = sources.history;
        let mut inputs = sources.inputs(Version::ONE);
        inputs.version = inputs.version + capitalized_price.version();
        inputs.len = inputs.len.min(capitalized_price.len());
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
        let (prices, timestamps) = (walk.prices(), walk.timestamps());
        if prices.len() < end || timestamps.len() < end {
            return Err(Error::Internal("entry ahead of the walk's prices and times"));
        }
        // The cohorts stand where the last update left them if it ended at `start` on this chain.
        let (mut live, reuse) = match live {
            Some(live)
                if live.version == version
                    && live.entries.len() == start
                    && history.follows(start, live.hash)? =>
            {
                (live, true)
            }
            _ => (
                LiveState {
                    cohorts: ByEntry::from_fn(|_| CohortState::default()),
                    entries: Vec::new(),
                    hash: [0; 32],
                    version,
                },
                false,
            ),
        };
        let data_start = live.entries.len();
        let anchor_start = data_start.saturating_sub(1);
        let anchors = capitalized_price.collect_range_at(anchor_start, end - 1);
        debug_assert_eq!(anchors.len(), end - 1 - anchor_start);
        live.entries.extend((data_start..end).map(|h| {
            classify(
                prices[h],
                h.checked_sub(1)
                    .map_or(Cents::NAN, |p| anchors[p - anchor_start]),
            )
        }));
        if !reuse {
            let origins = history.state_at(start)?;
            for (h, &amount) in origins.amounts().iter().enumerate() {
                live.cohorts
                    .get_mut(live.entries[h])
                    .increment_snapshot(&CostBasisSnapshot::from_utxo(prices[h], &supply(amount)));
            }
            for state in live.cohorts.iter_mut() {
                state.finish_restore();
            }
            live.hash = origins.hash();
        }
        let mut diffs = history.diffs(start)?;
        for chunk in write_chunks(range) {
            for h in chunk.clone() {
                let price = prices[h];
                let diff = (diffs.advance()?).ok_or(Error::Internal("history short of entry"))?;
                live.cohorts
                    .get_mut(live.entries[h])
                    .receive_utxo(&supply(diff.created), price);
                for (origin, amount) in diff.spent() {
                    let origin = origin as usize;
                    let age = Age::new(timestamps[h], timestamps[origin]);
                    live.cohorts.get_mut(live.entries[origin]).send_utxo(
                        &supply(amount),
                        price,
                        prices[origin],
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
                            prices[origin],
                            &supply(amount),
                        ));
                }
                for (target, state) in self.cohorts.iter_mut().zip(live.cohorts.iter_mut()) {
                    target.push(state, price);
                    state.reset_block();
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
        if let Some(hash) = diffs.hash() {
            live.hash = hash;
        }
        self.live = Some(live);
        Ok(())
    }
}
