use bitview_cohort::ByAddrType;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Sats, TxInIndex, TxIndex, TypeIndex};
use rayon::{join, prelude::*};
use tracing::{debug, info};
use vecdb::{AnyVec, PcoVec, ReadableVec, VecIndex, unlikely};

use super::{
    super::{
        state::{AddrStates, UTXOStates},
        vecs::Vecs,
    },
    AddrReaders, ComputeContext, IndexToTxIndexBuf, TxInReaders, TxOutReaders,
};
use crate::{
    addr::AddrMetricsState,
    block::{
        AddrCache, TransferAddressCache, normalize_supply, process_inputs_lean, process_outputs,
        process_received, process_typed_sent,
    },
    compute::write::write,
};

/// Process all blocks from starting_height to last_height.
#[allow(clippy::too_many_arguments)]
pub fn process_chunk(
    vecs: &mut Vecs,
    utxo_states: &mut UTXOStates,
    addr_states: &mut AddrStates,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    input_values: &PcoVec<TxInIndex, Sats>,
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

    // Create reusable iterators and buffers for per-block reads
    let tx_heights = mappings.tx_heights.read();
    let mut txout_iters = TxOutReaders::new(indexer);
    let mut txin_iters = TxInReaders::new(
        input_values,
        &indexer.vecs().inputs.outpoint,
        &indexer.vecs().inputs.output_type,
        &indexer.vecs().inputs.type_index,
        &tx_heights,
    );
    let mut txout_to_tx_index_buf = IndexToTxIndexBuf::new();
    let mut txin_to_tx_index_buf = IndexToTxIndexBuf::new();

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

    debug!("creating AddrCache");
    let mut cache = AddrCache::default();
    debug!("AddrCache created, entering main loop");

    // Pre-truncate all stored vecs to starting_height (one-time).
    // This eliminates per-push truncation checks inside the block loop.
    {
        let start = starting_height.to_usize();
        vecs.cohorts
            .par_iter_vecs_mut()
            .chain(vecs.addrs.par_iter_height_mut())
            .try_for_each(|v| v.any_truncate_if_needed_at(start))?;
    }

    let mut transfer_addresses = TransferAddressCache::default();

    // Main block iteration
    for height in starting_height.to_usize()..=last_height.to_usize() {
        let height = Height::from(height);

        if unlikely(height.is_multiple_of(100)) {
            info!("Computing metrics at block {height}...");
        } else {
            debug!("Processing chain at {}...", height);
        }

        // Get block metadata from pre-collected vecs
        let offset = height.to_usize() - start_usize;
        let first_tx_index = height_to_first_tx_index_vec[offset];
        let tx_count = (height_to_first_tx_index_vec
            .get(offset + 1)
            .map_or(indexer.vecs().transactions.txid.len(), |i| i.to_usize())
            - first_tx_index.to_usize()) as u64;
        let first_txout_index = height_to_first_txout_index_vec[offset].to_usize();
        let output_count = height_to_first_txout_index_vec
            .get(offset + 1)
            .map_or(indexer.vecs().outputs.value.len(), |i| i.to_usize())
            - first_txout_index;
        let first_txin_index = height_to_first_txin_index_vec[offset].to_usize();
        let input_count = height_to_first_txin_index_vec
            .get(offset + 1)
            .map_or(indexer.vecs().inputs.outpoint.len(), |i| i.to_usize())
            - first_txin_index;
        let block_price = height_to_price_collected[offset];

        // Debug validation: verify context methods match pre-collected values
        debug_assert_eq!(ctx.price_at(height), block_price);

        // Get first address mappings for this height from pre-collected vecs
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

        state.reset_per_block();

        debug_assert!(input_count > 0);

        let (outputs_result, inputs_result) = {
            // Collect both sides concurrently, then load their shared addresses once.
            let (
                (txout_index_to_tx_index, txout_data_vec),
                (
                    txin_index_to_tx_index,
                    (input_values, input_prev_heights, input_output_types, input_type_indexes),
                ),
            ) = join(
                || {
                    let txout_index_to_tx_index = txout_to_tx_index_buf.build(
                        first_tx_index,
                        tx_count,
                        tx_index_to_output_count,
                    );
                    let txout_data_vec =
                        txout_iters.collect_block_outputs(first_txout_index, output_count);
                    (txout_index_to_tx_index, txout_data_vec)
                },
                || {
                    let txin_index_to_tx_index = txin_to_tx_index_buf.build(
                        first_tx_index,
                        tx_count,
                        tx_index_to_input_count,
                    );
                    let input_data = txin_iters.collect_block_inputs(
                        first_txin_index + 1,
                        input_count - 1,
                        height,
                    );
                    (txin_index_to_tx_index, input_data)
                },
            );

            cache.load_block_addresses(
                txout_data_vec
                    .iter()
                    .map(|data| (data.output_type, data.type_index))
                    .chain(
                        input_output_types
                            .iter()
                            .copied()
                            .zip(input_type_indexes.iter().copied()),
                    ),
                &first_addr_indexes,
                &vr,
                &vecs.addr_state,
            );
            let (outputs_result, inputs_result) = join(
                || process_outputs(txout_index_to_tx_index, txout_data_vec),
                || {
                    process_inputs_lean(
                        &txin_index_to_tx_index[1..],
                        input_values,
                        input_output_types,
                        input_type_indexes,
                        input_prev_heights,
                        block_price,
                        height_to_price_vec,
                    )
                },
            );
            (outputs_result, inputs_result)
        };

        // Update tx_count from the transaction-ordered output and input maps.
        cache.update_tx_counts(&outputs_result.received, inputs_result.tx_index_vecs);

        let mut transacted = outputs_result.transacted;
        let mut detailed = inputs_result.detailed;

        normalize_supply(height, &mut transacted, &mut detailed, height_to_price_vec);

        transfer_addresses.prepare(
            outputs_result
                .received
                .iter()
                .flat_map(|(ty, entries)| entries.keys().copied().map(move |index| (ty, index))),
        );

        // Process UTXO cohorts and Addr cohorts in parallel
        let (_, addr_result) = join(
            || {
                utxo_states.receive_details(&transacted, block_price);
                detailed.apply(utxo_states);
            },
            || -> Result<()> {
                let mut lookup = cache.as_lookup();

                process_received(
                    outputs_result.received,
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
                )
            },
        );
        addr_result?;

        let active_addr_count = state.activity.active();
        vecs.addrs.push_height(&state, active_addr_count);

        addr_states.push(&mut vecs.cohorts, &mut vecs.addrs.funded, block_price);
        vecs.cohorts.push(utxo_states, block_price);
        utxo_states
            .type_
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        utxo_states
            .amount_range
            .iter_mut()
            .for_each(|state| state.reset_single_iteration_values());
        addr_states.reset_block();
    }
    drop(vr);
    cache.flush_into(&mut vecs.addr_state)?;
    // Final write - always save changes for rollback support

    let _lock = exit.lock();
    // Write to disk (pure I/O) - save changes for rollback
    write(vecs, utxo_states, addr_states, last_height, final_chunk)?;
    if !final_chunk {
        vecs.flush()?;
    }

    Ok(())
}
