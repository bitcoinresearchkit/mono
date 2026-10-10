use std::ops::{ControlFlow, Range};

use super::write::write;
use crate::{
    Vecs,
    block::{DetailedSpends, normalize_supply, remove_overwritten},
    state::{Transacted, UTXOStates},
};
use bitview_distribution::readers::{BatchColumns, BlockBounds, Columns};
use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height};
use rayon::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn process_chunk(
    vecs: &mut Vecs,
    states: &mut UTXOStates,
    indexer: &Indexer,
    columns: &mut Columns<'_, false>,
    blocks: Range<usize>,
    prices: &[Cents],
    final_chunk: bool,
    exit: &Exit,
) -> Result<()> {
    {
        let _lock = exit.lock();
        vecs.cohorts
            .par_iter_vecs_mut()
            .try_for_each(|v| v.any_truncate_if_needed_at(blocks.start))?;
    }
    let last_height = Height::from(blocks.end - 1);
    let bounds = BlockBounds::new(indexer, blocks);
    columns.for_each_batch(&bounds, |batch, columns| {
        let BatchColumns {
            output_values: values,
            output_types: types,
            input_values: spent_values,
            input_heights: origins,
            input_types: spent_types,
            ..
        } = columns;
        for height in batch.blocks.clone() {
            let received = batch.block_outputs(height);
            let spent = batch.block_spends(height);
            let price = prices[height];
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
                detailed.add(value, ty, prices[usize::from(origin)], price);
            }
            normalize_supply(Height::from(height), &mut transacted);
            states.receive_details(&transacted, price);
            detailed.apply(states);
            remove_overwritten(Height::from(height), states, prices);
            vecs.cohorts.push(states, price);
            states
                .type_
                .iter_mut()
                .chain(states.amount_range.iter_mut())
                .for_each(|s| s.reset_single_iteration_values());
        }
        Ok(ControlFlow::Continue(()))
    })?;
    let _lock = exit.lock();
    write(vecs, states, last_height, final_chunk)?;
    if !final_chunk {
        vecs.flush()?;
    }
    Ok(())
}
