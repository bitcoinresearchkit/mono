use super::{ComputeContext, Workspace, write::write};
use crate::{
    Vecs,
    block::{DetailedSpends, normalize_supply},
    state::{Transacted, UTXOStates},
};
use bitview_plugin_distribution_common::readers::index_range;
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use rayon::{join, prelude::*};
use vecdb::{AnyVec, ReadableVec, VecIndex};
const BATCH_BLOCKS: usize = 16;

pub fn process_chunk(
    vecs: &mut Vecs,
    states: &mut UTXOStates,
    indexer: &Indexer,
    workspace: &mut Workspace<'_>,
    ctx: &ComputeContext<'_>,
    final_chunk: bool,
    exit: &Exit,
) -> Result<()> {
    let start = ctx.starting_height.to_usize();
    let end = ctx.last_height.to_usize() + 1;
    let outputs = &indexer.vecs().outputs;
    let inputs = &indexer.vecs().inputs;
    let first_outputs = outputs
        .first_txout_index
        .collect_range_at(start, (end + 1).min(outputs.first_txout_index.len()));
    let first_inputs = inputs
        .first_txin_index
        .collect_range_at(start, (end + 1).min(inputs.first_txin_index.len()));
    {
        let _lock = exit.lock();
        vecs.cohorts
            .par_iter_vecs_mut()
            .try_for_each(|v| v.any_truncate_if_needed_at(start))?;
    }
    for batch_start in (start..end).step_by(BATCH_BLOCKS) {
        let batch_end = (batch_start + BATCH_BLOCKS).min(end);
        let batch_outputs = index_range(
            &first_outputs,
            batch_start - start,
            batch_end - start,
            outputs.value.len(),
        );
        let batch_inputs = index_range(
            &first_inputs,
            batch_start - start,
            batch_end - start,
            inputs.outpoint.len(),
        );
        let (received, spent) = join(
            || {
                workspace
                    .outputs
                    .collect_outputs(batch_outputs.start, batch_outputs.len())
            },
            || {
                workspace.inputs.collect_inputs(
                    batch_inputs.start,
                    batch_inputs.len(),
                    Height::from(batch_start),
                )
            },
        );
        let (values, types, _) = received?;
        let (spent_values, origins, spent_types, _) = spent?;
        for height in batch_start..batch_end {
            let offset = height - start;
            let received = index_range(&first_outputs, offset, offset + 1, outputs.value.len());
            let spent = index_range(&first_inputs, offset, offset + 1, inputs.outpoint.len());
            let received = received.start - batch_outputs.start..received.end - batch_outputs.start;
            let spent = spent.start + 1 - batch_inputs.start..spent.end - batch_inputs.start;
            let price = ctx.price_at(Height::from(height));
            let mut transacted = Transacted::default();
            for (&value, &ty) in values[received.clone()].iter().zip(&types[received]) {
                transacted.iterate(value, ty);
            }
            let mut detailed = DetailedSpends::default();
            for ((&value, &ty), &origin) in spent_values[spent.clone()]
                .iter()
                .zip(&spent_types[spent.clone()])
                .zip(&origins[spent])
            {
                detailed.add(value, ty, ctx.price_at(origin), price);
            }
            normalize_supply(
                Height::from(height),
                &mut transacted,
                &mut detailed,
                ctx.height_to_price,
            );
            states.receive_details(&transacted, price);
            detailed.apply(states);
            vecs.cohorts.push(states, price);
            states
                .type_
                .iter_mut()
                .chain(states.amount_range.iter_mut())
                .for_each(|s| s.reset_single_iteration_values());
        }
    }
    let _lock = exit.lock();
    write(vecs, states, ctx.last_height, final_chunk)?;
    if !final_chunk {
        vecs.flush()?;
    }
    Ok(())
}
