use bitview_cohort::ByAddrType;
use bitview_plugin_distribution_common::readers::index_range;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, TxIndex, TypeIndex};
use rayon::{join, prelude::*};
use tracing::{debug, info};
use vecdb::{AnyVec, ReadableVec, VecIndex, unlikely};

use super::{
    super::{state::AddrStates, vecs::Vecs},
    AddrReaders, ComputeContext, Workspace,
};
use crate::{
    addr::AddrMetricsState,
    block::{
        TransferAddressCache, process_inputs, process_outputs, process_received, process_typed_sent,
    },
    compute::write::write,
};

const BATCH_BLOCKS: usize = 16;

/// Process all blocks from starting_height to last_height.
#[allow(clippy::too_many_arguments)]
pub fn process_chunk(
    vecs: &mut Vecs,
    addr_states: &mut AddrStates,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    workspace: &mut Workspace<'_>,
    ctx: &ComputeContext<'_>,
    final_chunk: bool,
    exit: &Exit,
) -> Result<()> {
    if ctx.starting_height > ctx.last_height {
        return Ok(());
    }
    let starting_height = ctx.starting_height;
    let last_height = ctx.last_height;

    let height_to_first_tx_index = &indexer.vecs().transactions.first_tx_index;
    let height_to_first_txout_index = &indexer.vecs().outputs.first_txout_index;
    let height_to_first_txin_index = &indexer.vecs().inputs.first_txin_index;
    let tx_index_to_output_count = &mappings.tx_index.output_count;
    let tx_index_to_input_count = &mappings.tx_index.input_count;

    let height_to_price_vec = ctx.height_to_price;

    let start_usize = starting_height.to_usize();
    let end_usize = last_height.to_usize() + 1;

    let height_to_first_tx_index_vec: Vec<TxIndex> = height_to_first_tx_index.collect_range_at(
        start_usize,
        (end_usize + 1).min(height_to_first_tx_index.len()),
    );
    let height_to_first_txout_index_vec: Vec<_> = height_to_first_txout_index.collect_range_at(
        start_usize,
        (end_usize + 1).min(height_to_first_txout_index.len()),
    );
    let height_to_first_txin_index_vec: Vec<_> = height_to_first_txin_index.collect_range_at(
        start_usize,
        (end_usize + 1).min(height_to_first_txin_index.len()),
    );
    let height_to_price_collected = &ctx.height_to_price[start_usize..end_usize];

    debug!("creating AddrReaders");
    let vr = AddrReaders::new(&vecs.addr_state);
    debug!("AddrReaders created");

    let Workspace {
        outputs,
        inputs,
        output_txs,
        input_txs,
        addresses: cache,
    } = workspace;

    // Pre-collect first address mappings per type for the block range
    let first_p2a_vec = indexer
        .vecs()
        .addrs
        .p2a
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2pk33_vec = indexer
        .vecs()
        .addrs
        .p2pk33
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2pk65_vec = indexer
        .vecs()
        .addrs
        .p2pk65
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2pkh_vec = indexer
        .vecs()
        .addrs
        .p2pkh
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2sh_vec = indexer
        .vecs()
        .addrs
        .p2sh
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2tr_vec = indexer
        .vecs()
        .addrs
        .p2tr
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2wpkh_vec = indexer
        .vecs()
        .addrs
        .p2wpkh
        .first_index
        .collect_range_at(start_usize, end_usize);
    let first_p2wsh_vec = indexer
        .vecs()
        .addrs
        .p2wsh
        .first_index
        .collect_range_at(start_usize, end_usize);

    debug!(
        "recovering addr metrics state from height {}",
        starting_height
    );
    let mut state = AddrMetricsState::from((&vecs.addrs, starting_height));
    debug!("addr metrics state recovered");

    // Pre-truncate all stored vecs to starting_height (one-time).
    // This eliminates per-push truncation checks inside the block loop.
    {
        let _lock = exit.lock();
        let start = starting_height.to_usize();
        vecs.balances
            .par_iter_vecs_mut()
            .chain(vecs.addrs.par_iter_height_mut())
            .try_for_each(|v| v.any_truncate_if_needed_at(start))?;
    }

    let mut transfer_addresses = TransferAddressCache::default();

    // Bound source buffers while loading cold address state once for several blocks.
    for batch_start in (start_usize..end_usize).step_by(BATCH_BLOCKS) {
        let batch_end = (batch_start + BATCH_BLOCKS).min(end_usize);
        let offset = batch_start - start_usize;
        let end_offset = batch_end - start_usize;
        let batch_outputs = index_range(
            &height_to_first_txout_index_vec,
            offset,
            end_offset,
            indexer.vecs().outputs.value.len(),
        );
        let batch_inputs = index_range(
            &height_to_first_txin_index_vec,
            offset,
            end_offset,
            indexer.vecs().inputs.outpoint.len(),
        );
        let (output_columns, input_columns) = join(
            || outputs.collect_outputs(batch_outputs.start, batch_outputs.len()),
            || {
                inputs.collect_inputs(
                    batch_inputs.start,
                    batch_inputs.len(),
                    Height::from(batch_start),
                )
            },
        );
        let (output_values, output_types, output_indexes) = output_columns?;
        let (input_values, input_heights, input_types, input_indexes) = input_columns?;

        // Addresses created within this batch start at these per-type indexes.
        let first_addr_indexes = ByAddrType {
            p2a: TypeIndex::from(first_p2a_vec[offset].to_usize()),
            p2pk33: TypeIndex::from(first_p2pk33_vec[offset].to_usize()),
            p2pk65: TypeIndex::from(first_p2pk65_vec[offset].to_usize()),
            p2pkh: TypeIndex::from(first_p2pkh_vec[offset].to_usize()),
            p2sh: TypeIndex::from(first_p2sh_vec[offset].to_usize()),
            p2tr: TypeIndex::from(first_p2tr_vec[offset].to_usize()),
            p2wpkh: TypeIndex::from(first_p2wpkh_vec[offset].to_usize()),
            p2wsh: TypeIndex::from(first_p2wsh_vec[offset].to_usize()),
        };

        cache.load_addresses(
            output_types
                .iter()
                .copied()
                .zip(output_indexes.iter().copied())
                .chain(
                    input_types
                        .iter()
                        .copied()
                        .zip(input_indexes.iter().copied()),
                ),
            &first_addr_indexes,
            &vr,
            &vecs.addr_state,
        );

        for height in batch_start..batch_end {
            let height = Height::from(height);
            if unlikely(height.is_multiple_of(100)) {
                info!("Computing metrics at block {height}...");
            } else {
                debug!("Processing chain at {height}...");
            }
            let offset = height.to_usize() - start_usize;
            let transactions = index_range(
                &height_to_first_tx_index_vec,
                offset,
                offset + 1,
                indexer.vecs().transactions.txid.len(),
            );
            let outputs = index_range(
                &height_to_first_txout_index_vec,
                offset,
                offset + 1,
                indexer.vecs().outputs.value.len(),
            );
            let inputs = index_range(
                &height_to_first_txin_index_vec,
                offset,
                offset + 1,
                indexer.vecs().inputs.outpoint.len(),
            );
            let block_price = height_to_price_collected[offset];
            debug_assert_eq!(ctx.price_at(height), block_price);
            let output_range =
                outputs.start - batch_outputs.start..outputs.end - batch_outputs.start;
            // Omit this block's coinbase, including coinbase-only blocks.
            let input_range =
                inputs.start + 1 - batch_inputs.start..inputs.end - batch_inputs.start;
            state.reset_per_block();

            debug_assert!(!inputs.is_empty());

            let (outputs_result, inputs_result) = join(
                || {
                    process_outputs(
                        output_txs.build(
                            TxIndex::from(transactions.start),
                            transactions.len() as u64,
                            tx_index_to_output_count,
                        ),
                        &output_values[output_range.clone()],
                        &output_types[output_range.clone()],
                        &output_indexes[output_range],
                    )
                },
                || {
                    process_inputs(
                        input_txs
                            .build(
                                TxIndex::from(transactions.start),
                                transactions.len() as u64,
                                tx_index_to_input_count,
                            )
                            .skip(1),
                        &input_values[input_range.clone()],
                        &input_types[input_range.clone()],
                        &input_indexes[input_range.clone()],
                        &input_heights[input_range],
                    )
                },
            );

            // Update tx_count from the transaction-ordered output and input maps.
            cache.update_tx_counts(&outputs_result, inputs_result.tx_index_vecs);

            transfer_addresses.prepare(
                outputs_result.iter().flat_map(|(ty, entries)| {
                    entries.keys().copied().map(move |index| (ty, index))
                }),
            );

            let mut lookup = cache.as_lookup();
            process_received(
                outputs_result,
                addr_states,
                &mut lookup,
                block_price,
                &mut state,
            );
            process_typed_sent(
                inputs_result.sent_data.into_typed(height_to_price_vec),
                addr_states,
                &mut lookup,
                block_price,
                &mut state,
                &mut transfer_addresses,
            )?;

            let active_addr_count = state.activity.active();
            vecs.addrs.push_height(&state, active_addr_count);

            addr_states.push(&mut vecs.balances, &mut vecs.addrs.funded, block_price);
            addr_states.reset_block();
        }
    }
    drop(vr);
    let _lock = exit.lock();
    cache.flush_into(&mut vecs.addr_state)?;
    write(vecs, addr_states, last_height, final_chunk)?;
    if !final_chunk {
        vecs.flush()?;
    }

    Ok(())
}
