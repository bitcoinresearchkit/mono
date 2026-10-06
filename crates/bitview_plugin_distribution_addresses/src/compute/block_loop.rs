use std::ops::{ControlFlow, Range};

use bitview_cohort::ByAddrType;
use bitview_plugin_distribution_common::readers::{BatchColumns, BlockBounds, index_range};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{FundedAddrData, TypeIndex};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, OutputType, Sats, TxIndex};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::{debug, info};
use vecdb::{AnyVec, ReadableVec, VecIndex, unlikely};

use super::{AddrReaders, Workspace, readers::tx_indexes};
use crate::{
    Vecs,
    addr::{AddrMetricsState, SHARDS, SourcedAddrData},
    block::{
        AddrTypeLookup, Received, TransferAddressCache, TxIndexes, process_inputs, process_outputs,
        process_received, process_sent,
    },
    compute::write::write,
    state::{AddrStates, CohortLog},
};

/// One address type's activity in one block, for one shard of its addresses.
struct TypeBlock {
    received: FxHashMap<TypeIndex, Received>,
    /// Spends in the block's established order, with their creation prices.
    spends: Vec<(TypeIndex, Sats, Cents)>,
    /// Transactions each address spent in.
    sent_txs: FxHashMap<TypeIndex, TxIndexes>,
}

/// Flush the address cache early once it holds this many addresses (checked after each batch):
/// it bounds the cache's tables (about 57 bytes a slot) however busy the chain gets. An
/// addresses-only run to 970,056 blocks peaked at 7.6 GiB with a third fewer flushes than a 16M
/// bound.
const MAX_CACHED_ADDRS: usize = 32_000_000;

/// Process `blocks` until the address cache is full, flush, and return the next height.
#[allow(clippy::too_many_arguments)]
pub fn process_chunk(
    vecs: &mut Vecs,
    addr_states: &mut AddrStates,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    workspace: &mut Workspace<'_>,
    blocks: Range<usize>,
    prices: &[Cents],
    last_chunk: bool,
    exit: &Exit,
) -> Result<usize> {
    let starting_height = Height::from(blocks.start);
    let start_usize = blocks.start;
    let end_usize = blocks.end;

    let height_to_first_tx_index = &indexer.vecs().transactions.first_tx_index;
    let tx_index_to_output_count = &mappings.tx_index.output_count;
    let tx_index_to_input_count = &mappings.tx_index.input_count;

    let height_to_first_tx_index_vec: Vec<TxIndex> = height_to_first_tx_index.collect_range_at(
        start_usize,
        (end_usize + 1).min(height_to_first_tx_index.len()),
    );

    debug!("creating AddrReaders");
    let vr = AddrReaders::new(&vecs.addr_state);
    debug!("AddrReaders created");

    let Workspace {
        columns,
        addresses: cache,
    } = workspace;
    let tx_count = indexer.vecs().transactions.txid.len();

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

    // Load cold address state once for each batch of blocks.
    let bounds = BlockBounds::new(indexer, blocks);
    let mut next = end_usize;
    columns.for_each_batch(&bounds, |batch, columns| {
        let offset = batch.blocks.start - start_usize;
        let BatchColumns {
            output_values,
            output_types,
            output_indexes,
            input_values,
            input_heights,
            input_types,
            input_indexes,
        } = columns;

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

        // Group each block's outputs and inputs by address, blocks in parallel.
        let groups = batch
            .blocks
            .clone()
            .into_par_iter()
            .map(|height| {
                let offset = height - start_usize;
                let transactions =
                    index_range(&height_to_first_tx_index_vec, offset, offset + 1, tx_count);
                let first_tx = TxIndex::from(transactions.start);
                let tx_len = transactions.len() as u64;
                let output_range = batch.block_outputs(height);
                // Omit this block's coinbase, including coinbase-only blocks.
                let input_range = batch.block_spends(height);
                let received = process_outputs(
                    tx_indexes(first_tx, tx_len, tx_index_to_output_count),
                    &output_values[output_range.clone()],
                    &output_types[output_range.clone()],
                    &output_indexes[output_range],
                );
                let inputs = process_inputs(
                    tx_indexes(first_tx, tx_len, tx_index_to_input_count).skip(1),
                    &input_values[input_range.clone()],
                    &input_types[input_range.clone()],
                    &input_indexes[input_range.clone()],
                    &input_heights[input_range],
                );
                (
                    received,
                    inputs.sent_data.into_typed(prices),
                    inputs.tx_index_vecs,
                )
            })
            .collect::<Vec<_>>();
        let mut by_shard: [ByAddrType<Vec<TypeBlock>>; SHARDS] = Default::default();
        for (received, spends, sent_txs) in groups {
            for (((by_type, received), spends), sent_txs) in
                by_shard.iter_mut().zip(received).zip(spends).zip(sent_txs)
            {
                for (((output_type, received), (spends_type, spends)), (sent_type, sent_txs)) in
                    received
                        .into_iter()
                        .zip(spends.into_iter())
                        .zip(sent_txs.into_iter())
                {
                    debug_assert!(output_type == spends_type && output_type == sent_type);
                    by_type.get_mut_unwrap(output_type).push(TypeBlock {
                        received,
                        spends,
                        sent_txs,
                    });
                }
            }
        }

        // Each address touches only its own state, so every shard of every type runs apart from
        // the same base metrics; cohort changes are logged per block and applied below in order.
        let block_prices = &prices[batch.blocks.clone()];
        let mut base = state.clone();
        base.reset_per_block();
        let base = &base;
        let processed = cache
            .shards_mut()
            .zip(by_shard.into_iter().flat_map(ByAddrType::into_iter))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|((output_type, addrs), (blocks_type, blocks))| {
                debug_assert_eq!(output_type, blocks_type);
                process_shard(output_type, addrs, blocks, block_prices, base)
                    .map(|blocks| (output_type, blocks))
            })
            .collect::<Result<Vec<_>>>()?;

        for (offset, height) in batch.blocks.clone().enumerate() {
            let height = Height::from(height);
            if unlikely(height.is_multiple_of(100)) {
                info!("Computing metrics at block {height}...");
            } else {
                debug!("Processing chain at {height}...");
            }
            state.clone_from(base);
            for (output_type, blocks) in &processed {
                let (cohorts, metrics) = &blocks[offset];
                cohorts.apply_to(&mut addr_states.amount_range);
                state.add_type_delta(metrics, base, *output_type);
            }
            vecs.addrs.push_height(&state);
            addr_states.push(
                &mut vecs.balances,
                &mut vecs.addrs.funded,
                block_prices[offset],
            );
            addr_states.reset_block();
        }
        if cache.len() >= MAX_CACHED_ADDRS {
            next = batch.blocks.end;
            return Ok(ControlFlow::Break(()));
        }
        Ok(ControlFlow::Continue(()))
    })?;
    drop(vr);
    let last_height = Height::from(next - 1);
    let _lock = exit.lock();
    cache.flush_into(&mut vecs.addr_state)?;
    // Every write of the range nearest the tip keeps its changes, so a reorg can roll back across early flushes.
    write(vecs, addr_states, last_height, last_chunk)?;
    if !(last_chunk && next == end_usize) {
        vecs.flush()?;
    }

    Ok(next)
}

/// Apply one shard of an address type's blocks in order: per block its transaction counts,
/// receives, then spends. Returns each block's cohort changes and the metrics after it, of
/// which only this type's values are meaningful.
fn process_shard(
    output_type: OutputType,
    addrs: &mut FxHashMap<TypeIndex, SourcedAddrData<FundedAddrData>>,
    blocks: Vec<TypeBlock>,
    prices: &[Cents],
    base: &AddrMetricsState,
) -> Result<Vec<(CohortLog, AddrMetricsState)>> {
    let mut metrics = base.clone();
    let mut lookup = AddrTypeLookup::new(addrs);
    let mut transfers = TransferAddressCache::default();
    let mut processed = Vec::with_capacity(blocks.len());
    for (
        TypeBlock {
            received,
            spends,
            sent_txs,
        },
        &price,
    ) in blocks.into_iter().zip(prices)
    {
        metrics.reset_per_block();
        lookup.update_tx_counts(&received, sent_txs);
        transfers.prepare(received.keys().copied());
        let mut cohorts = CohortLog::default();
        let mut type_metrics = metrics.select(output_type);
        process_received(
            received,
            &mut cohorts,
            &mut lookup,
            price,
            &mut type_metrics,
        );
        process_sent(
            &spends,
            &mut cohorts,
            &mut lookup,
            price,
            &mut type_metrics,
            &mut transfers,
        )?;
        processed.push((cohorts, metrics.clone()));
    }
    Ok(processed)
}
