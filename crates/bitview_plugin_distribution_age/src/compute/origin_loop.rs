use super::{ComputeContext, origin_targets::OriginTargets};
use crate::state::{UTXOStates, supply, tick_tock_next_block};
use bitview_cohort::EntryPrice;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredF64};
use statedb::Cursor;

pub fn replay_origins(
    vecs: &mut OriginTargets<'_>,
    states: &mut UTXOStates,
    ctx: &ComputeContext<'_>,
    entries: &mut Vec<EntryPrice>,
    mut anchor: Cents,
    cursor: &mut Cursor<'_>,
    mut on_block: impl FnMut(Height, &UTXOStates) -> Result<()>,
) -> Result<Option<Height>> {
    states.init_fenwick_if_needed();
    let mut min_modified: Option<Height> = None;

    for h in usize::from(ctx.starting_height)..=usize::from(ctx.last_height) {
        let height = Height::from(h);
        let timestamp = ctx.height_to_timestamp[h];
        let price = ctx.height_to_price[h];

        let tick = tick_tock_next_block(states, cursor.state().amounts(), ctx, timestamp);
        let diff = cursor.advance()?.expect("validated origin range");
        let spent = diff.removed();
        let created = supply(diff.created);
        let entry = EntryPrice::from_is_discount(anchor == Cents::ZERO || price <= anchor);
        entries.push(entry);
        let satblocks: u128 = spent
            .iter()
            .map(|(origin, supply)| (h - *origin as usize) as u128 * u128::from(supply.sats))
            .sum();
        vecs.coinblocks_destroyed.push_block(StoredF64::from(
            satblocks as f64 / Sats::ONE_BTC_U128 as f64,
        ));
        vecs.cohorts.supply.push_maturation(&tick.matured, price);
        for (target, value) in vecs
            .coindays_created
            .iter_mut()
            .zip(tick.coindays_created.iter())
        {
            target.push_block(*value);
        }
        states.receive_origins(created, height, timestamp, price, entry);
        if let Some(h) = states.send_origins(
            spent.iter().map(|&(h, v)| (Height::new(h), supply(v))),
            height,
            entries,
            ctx,
        ) {
            min_modified = Some(min_modified.map_or(h, |old| old.min(h)));
        }
        states.update_fenwick_from_pending();
        states.apply_pending();

        let unrealized = vecs.cohorts.push_supply_and_unrealized(states, price);
        vecs.cohorts.push_outputs(states);
        vecs.cohorts.push_activity(states, price);
        vecs.cohorts.push_realized(states);
        anchor = vecs.cohorts.push_aggregate(states, price, &unrealized);
        states.reset_block();
        vecs.cohorts.push_aggregate_percentiles(states, price);
        on_block(height, states)?;
    }

    Ok(min_modified)
}
