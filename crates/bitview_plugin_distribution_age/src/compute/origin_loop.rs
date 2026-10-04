use bitview_primitives::CoinBlocks;
use brk_error::Result;
use brk_types::{Height, Sats};
use statedb::Cursor;

use super::{ComputeContext, origin_targets::OriginTargets};
use crate::state::{UTXOStates, supply, tick_tock_next_block};

pub fn replay_origins(
    vecs: &mut OriginTargets<'_>,
    states: &mut UTXOStates,
    ctx: &ComputeContext<'_>,
    cursor: &mut Cursor<'_>,
    mut on_block: impl FnMut(Height, &UTXOStates),
) -> Result<()> {
    for h in usize::from(ctx.starting_height)..=usize::from(ctx.last_height) {
        let height = Height::from(h);
        let timestamp = ctx.height_to_timestamp[h];
        let price = ctx.height_to_price[h];

        let tick = tick_tock_next_block(states, cursor.state().amounts(), ctx, timestamp);
        let diff = cursor.advance()?.expect("validated origin range");
        let spent = diff.removed();
        let created = supply(diff.created);
        vecs.cohorts.supply.push_maturation(&tick.matured, price);
        for (target, value) in vecs
            .coindays_created
            .iter_mut()
            .zip(tick.coindays_created.iter())
        {
            target.push_block(*value);
        }
        states.receive_origins(created, height, timestamp, price);
        let satblocks =
            states.send_origins(spent.map(|(h, v)| (Height::new(h), supply(v))), height, ctx);
        vecs.coinblocks_destroyed.push_block(CoinBlocks::from(
            satblocks as f64 / Sats::ONE_BTC_U128 as f64,
        ));

        states.apply_pending();

        vecs.cohorts.push_supply_and_unrealized(states, price);
        vecs.cohorts.push_outputs(states);
        vecs.cohorts.push_activity(states, price);
        vecs.cohorts.push_realized(states);

        states.reset_block();

        on_block(height, states);
    }

    Ok(())
}
