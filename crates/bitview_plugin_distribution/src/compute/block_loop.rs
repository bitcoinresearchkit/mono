use brk_error::Result;

use std::time::{Duration, Instant};

use bitview_cohort::{ByAddrType, EntryPrice, Filter, Term};
use bitview_plugin_indexer::Indexer;
use brk_types::{
    Cents, Date, Height, ONE_DAY_IN_SEC, OutputType, RangeMap, Sats, StoredF64, Timestamp, TxIndex,
    TypeIndex,
};
use rayon::prelude::*;
use tracing::{debug, info};
use vecdb::{AnyVec, Exit, ReadableVec, VecIndex, unlikely};

use crate::{
    addr::{AddrMetricsState, FundedAddrCountsVecs},
    block::{
        AddrCache, TransferAddressCache, process_inputs, process_outputs, process_received,
        process_sent,
    },
    compute::write::write,
    state::{BlockState, Transacted},
};

use super::{
    super::{
        metrics::CohortMetrics,
        state::{AddrStates, UTXOStates},
        vecs::Vecs,
    },
    AddrReaders, BIP30_DUPLICATE_HEIGHT_1, BIP30_DUPLICATE_HEIGHT_2, BIP30_ORIGINAL_HEIGHT_1,
    BIP30_ORIGINAL_HEIGHT_2, ComputeContext, IndexToTxIndexBuf, PriceRangeMax, TxInReaders,
    TxOutReaders,
};

const FLUSH_BLOCK_INTERVAL: usize = 10_000;
const FLUSH_TIME_INTERVAL: Duration = Duration::from_secs(60);

fn is_periodic_flush_due(
    height: Height,
    last_height: Height,
    pending_blocks: usize,
    elapsed: Duration,
) -> bool {
    height != last_height
        && (pending_blocks >= FLUSH_BLOCK_INTERVAL
            || (height != Height::ZERO
                && height.to_usize().is_multiple_of(FLUSH_BLOCK_INTERVAL)
                && elapsed >= FLUSH_TIME_INTERVAL))
}

/// Process all blocks from starting_height to last_height.
#[allow(clippy::too_many_arguments)]
pub fn process_blocks(
    vecs: &mut Vecs,
    utxo_states: &mut UTXOStates,
    addr_states: &mut AddrStates,
    indexer: &Indexer,
    mappings: &bitview_plugin_mappings::Vecs,
    inputs: &bitview_plugin_inputs::Vecs,
    outputs: &bitview_plugin_outputs::Vecs,
    transactions: &bitview_plugin_transactions::Vecs,
    starting_height: Height,
    last_height: Height,
    chain_state: &mut Vec<BlockState>,
    tx_index_to_height: &mut RangeMap<TxIndex, Height>,
    mut entry_anchor: Cents,
    cached_prices: &[Cents],
    cached_timestamps: &[Timestamp],
    cached_price_range_max: &PriceRangeMax,
    exit: &Exit,
) -> Result<()> {
    let ctx = ComputeContext {
        starting_height,
        last_height,
        height_to_timestamp: cached_timestamps,
        height_to_price: cached_prices,
        price_range_max: cached_price_range_max,
    };

    if ctx.starting_height > ctx.last_height {
        return Ok(());
    }

    let height_to_first_tx_index = &indexer.vecs().transactions.first_tx_index;
    let height_to_first_txout_index = &indexer.vecs().outputs.first_txout_index;
    let height_to_first_txin_index = &indexer.vecs().inputs.first_txin_index;
    let height_to_tx_count = &transactions.count.total.block;
    let height_to_output_count = &outputs.count.total.sum;
    let height_to_input_count = &inputs.count.sum;
    let tx_index_to_output_count = &mappings.tx_index.output_count;
    let tx_index_to_input_count = &mappings.tx_index.input_count;

    let height_to_price_vec = cached_prices;

    let start_usize = starting_height.to_usize();
    let end_usize = last_height.to_usize() + 1;

    let height_to_first_tx_index_vec: Vec<TxIndex> =
        height_to_first_tx_index.collect_range_at(start_usize, end_usize);
    let height_to_first_txout_index_vec: Vec<_> =
        height_to_first_txout_index.collect_range_at(start_usize, end_usize);
    let height_to_first_txin_index_vec: Vec<_> =
        height_to_first_txin_index.collect_range_at(start_usize, end_usize);
    let height_to_tx_count_vec: Vec<_> =
        height_to_tx_count.collect_range_at(start_usize, end_usize);
    let height_to_output_count_vec: Vec<_> =
        height_to_output_count.collect_range_at(start_usize, end_usize);
    let height_to_input_count_vec: Vec<_> =
        height_to_input_count.collect_range_at(start_usize, end_usize);
    let height_to_timestamp_collected = &cached_timestamps[start_usize..end_usize];
    let height_to_price_collected = &cached_prices[start_usize..end_usize];

    // Pre-compute day boundaries to avoid per-block division in the hot loop
    let is_last_of_day: Vec<bool> = (start_usize..end_usize)
        .map(|h| {
            h == end_usize - 1
                || *cached_timestamps[h] / ONE_DAY_IN_SEC
                    != *cached_timestamps[h + 1] / ONE_DAY_IN_SEC
        })
        .collect();

    debug!("creating AddrReaders");
    let mut vr = AddrReaders::new(&vecs.addr_state);
    debug!("AddrReaders created");

    // Extend tx_index_to_height RangeMap with new entries (incremental, O(new_blocks))
    let target_len = indexer.vecs().transactions.first_tx_index.len();
    let current_len = tx_index_to_height.len();
    if current_len < target_len {
        debug!(
            "extending tx_index_to_height RangeMap from {} to {}",
            current_len, target_len
        );
        let new_entries: Vec<TxIndex> = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_range_at(current_len, target_len);
        for first_tx_index in new_entries {
            tx_index_to_height.push(first_tx_index);
        }
    } else if current_len > target_len {
        debug!(
            "truncating tx_index_to_height RangeMap from {} to {}",
            current_len, target_len
        );
        tx_index_to_height.truncate(target_len);
    }
    debug!(
        "tx_index_to_height RangeMap ready ({} entries)",
        tx_index_to_height.len()
    );

    // Create reusable iterators and buffers for per-block reads
    let mut txout_iters = TxOutReaders::new(indexer);
    let mut txin_iters = TxInReaders::new(indexer, &inputs.value, tx_index_to_height);
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

    // Initialize Fenwick tree from imported BTreeMap state (one-time)
    utxo_states.init_fenwick_if_needed(&Filter::Term(Term::Sth));

    // Pre-truncate all stored vecs to starting_height (one-time).
    // This eliminates per-push truncation checks inside the block loop.
    {
        let start = starting_height.to_usize();
        vecs.cohorts
            .par_iter_vecs_mut()
            .chain(vecs.addrs.par_iter_height_mut())
            .chain(
                [
                    vecs.coindays_created.stored_mut(),
                    vecs.coinblocks_destroyed.stored_mut(),
                ]
                .into_par_iter(),
            )
            .try_for_each(|v| v.any_truncate_if_needed_at(start))?;
    }

    let mut transfer_addresses = TransferAddressCache::default();

    // Track earliest chain_state modification from sends (for incremental supply_state writes)
    let mut min_supply_modified: Option<Height> = None;
    let mut pending_blocks = 0;
    let mut last_flush = Instant::now();

    // Main block iteration
    for height in starting_height.to_usize()..=last_height.to_usize() {
        let height = Height::from(height);

        if unlikely(height.is_multiple_of(100)) {
            info!("Processing chain at {}...", height);
        } else {
            debug!("Processing chain at {}...", height);
        }

        // Get block metadata from pre-collected vecs
        let offset = height.to_usize() - start_usize;
        let first_tx_index = height_to_first_tx_index_vec[offset];
        let tx_count = u64::from(height_to_tx_count_vec[offset]);
        let first_txout_index = height_to_first_txout_index_vec[offset].to_usize();
        let output_count = u64::from(height_to_output_count_vec[offset]) as usize;
        let first_txin_index = height_to_first_txin_index_vec[offset].to_usize();
        let input_count = u64::from(height_to_input_count_vec[offset]) as usize;
        let timestamp = height_to_timestamp_collected[offset];
        let block_price = height_to_price_collected[offset];

        // Debug validation: verify context methods match pre-collected values
        debug_assert_eq!(ctx.timestamp_at(height), timestamp);
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

        // Keep tick-tock concurrent with the block reads and address processing.
        let (tick_tock, (outputs_result, inputs_result)) = rayon::join(
            || crate::state::tick_tock_next_block(utxo_states, chain_state, timestamp),
            || {
                // Collect both sides concurrently, then load their shared addresses once.
                let (
                    (txout_index_to_tx_index, txout_data_vec),
                    (
                        txin_index_to_tx_index,
                        (input_values, input_prev_heights, input_output_types, input_type_indexes),
                    ),
                ) = rayon::join(
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

                rayon::join(
                    || process_outputs(txout_index_to_tx_index, txout_data_vec),
                    || {
                        process_inputs(
                            &txin_index_to_tx_index[1..],
                            input_values,
                            input_output_types,
                            input_type_indexes,
                            input_prev_heights,
                        )
                    },
                )
            },
        );

        // Update tx_count from the transaction-ordered output and input maps.
        cache.update_tx_counts(outputs_result.tx_index_vecs, inputs_result.tx_index_vecs);

        let mut transacted = outputs_result.transacted;
        let mut height_to_sent = inputs_result.height_to_sent;

        // Handle special cases
        if height == Height::ZERO {
            // Genesis block - reset transacted (50 BTC is unspendable, handled in supply module)
            transacted = Transacted::default();
        } else if height == Height::new(BIP30_DUPLICATE_HEIGHT_1)
            || height == Height::new(BIP30_DUPLICATE_HEIGHT_2)
        {
            // BIP30: Add 50 BTC to spent from original height
            let original_height = if height == Height::new(BIP30_DUPLICATE_HEIGHT_1) {
                Height::new(BIP30_ORIGINAL_HEIGHT_1)
            } else {
                Height::new(BIP30_ORIGINAL_HEIGHT_2)
            };
            height_to_sent
                .entry(original_height)
                .or_default()
                .iterate(Sats::FIFTY_BTC, OutputType::P2PK65);
        }

        let entry = EntryPrice::from_is_discount(
            entry_anchor == Cents::ZERO || block_price <= entry_anchor,
        );

        // Push current block state before processing cohort updates
        chain_state.push(BlockState {
            supply: transacted.spendable_supply,
            entry,
            price: block_price,
            timestamp,
        });

        // Compute total coinblocks destroyed (once globally, before send() consumes height_to_sent)
        {
            let h = height.to_usize();
            let total_satblocks: u128 = height_to_sent
                .iter()
                .filter(|(rh, _)| rh.to_usize() < h)
                .map(|(rh, sent)| {
                    let blocks_old = h - rh.to_usize();
                    blocks_old as u128 * u64::from(sent.spendable_supply.value) as u128
                })
                .sum();
            vecs.coinblocks_destroyed.push_block(StoredF64::from(
                total_satblocks as f64 / Sats::ONE_BTC_U128 as f64,
            ));
        }

        // Record maturation (sats crossing age boundaries)
        vecs.cohorts
            .supply
            .push_maturation(&tick_tock.matured, block_price);
        vecs.coindays_created.push_block(tick_tock.coindays_created);

        transfer_addresses.prepare(&outputs_result.received_data);

        // Process UTXO cohorts and Addr cohorts in parallel
        let (_, addr_result) = rayon::join(
            || {
                // UTXO cohorts receive/send
                utxo_states.receive(transacted, height, timestamp, block_price, entry);
                if let Some(min_h) =
                    utxo_states.send(height_to_sent, chain_state, ctx.price_range_max)
                {
                    min_supply_modified =
                        Some(min_supply_modified.map_or(min_h, |cur| cur.min(min_h)));
                }
            },
            || -> Result<()> {
                let mut lookup = cache.as_lookup();

                process_received(
                    outputs_result.received_data,
                    addr_states,
                    &mut lookup,
                    block_price,
                    &mut state,
                );

                process_sent(
                    inputs_result.sent_data,
                    addr_states,
                    &mut lookup,
                    block_price,
                    &mut state,
                    &mut transfer_addresses,
                    height_to_price_vec,
                )
            },
        );
        addr_result?;

        // Update Fenwick tree from pending deltas (must happen before push_cohort_states drains pending)
        utxo_states.update_fenwick_from_pending();

        let active_addr_count = state.activity.active();
        vecs.addrs.push_height(&state, active_addr_count);

        let is_last_of_day = is_last_of_day[offset];
        let date_opt = is_last_of_day.then(|| Date::from(timestamp));

        entry_anchor = push_cohort_states(
            &mut vecs.cohorts,
            &mut vecs.addrs.funded,
            utxo_states,
            addr_states,
            height,
            block_price,
        );

        vecs.cohorts.push_aggregate_percentiles(
            utxo_states,
            block_price,
            date_opt,
            &vecs.states_path,
        )?;

        pending_blocks += 1;

        // Periodic checkpoint flush
        if is_periodic_flush_due(height, last_height, pending_blocks, last_flush.elapsed()) {
            // Drop readers to release mmap handles
            drop(vr);

            cache.flush_into(&mut vecs.addr_state)?;

            let _lock = exit.lock();

            // Write to disk (pure I/O) - no changes saved for periodic flushes
            write(
                vecs,
                utxo_states,
                addr_states,
                height,
                chain_state,
                min_supply_modified,
                false,
            )?;
            min_supply_modified = None;
            crate::vecs::flush(vecs)?;
            pending_blocks = 0;
            last_flush = Instant::now();

            // Recreate readers
            vr = AddrReaders::new(&vecs.addr_state);
        }
    }

    // Final write - always save changes for rollback support

    let _lock = exit.lock();
    drop(vr);

    cache.flush_into(&mut vecs.addr_state)?;

    // Write to disk (pure I/O) - save changes for rollback
    write(
        vecs,
        utxo_states,
        addr_states,
        last_height,
        chain_state,
        min_supply_modified,
        true,
    )?;

    Ok(())
}

/// Push cohort states to height-indexed vectors, then reset per-block values.
fn push_cohort_states(
    cohorts: &mut CohortMetrics,
    funded_addr_counts: &mut FundedAddrCountsVecs,
    utxo_states: &mut UTXOStates,
    addr_states: &mut AddrStates,
    height: Height,
    height_price: Cents,
) -> Cents {
    // Phase 1: finish state updates before metric-first sources read them.
    utxo_states.apply_pending();
    addr_states.push(cohorts, funded_addr_counts, height, height_price);

    // Phase 2: push the typed supply matrices, then aggregate age-range states.
    let unrealized_states = cohorts.push_supply_and_unrealized(utxo_states, height_price);
    cohorts.push_outputs(utxo_states);
    cohorts.push_activity(utxo_states, height_price);
    cohorts.push_realized(utxo_states);
    let all_capitalized_price =
        cohorts.push_overlapping(utxo_states, height_price, &unrealized_states);

    // Phase 3: reset per-block values
    utxo_states.reset_block();
    addr_states.reset_block();

    all_capitalized_price
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periodic_flush_uses_block_limit_or_aligned_time() {
        let checkpoint_height = Height::from(FLUSH_BLOCK_INTERVAL);

        assert!(!is_periodic_flush_due(
            Height::ZERO,
            checkpoint_height,
            0,
            FLUSH_TIME_INTERVAL
        ));
        assert!(!is_periodic_flush_due(
            checkpoint_height.incremented(),
            checkpoint_height.incremented().incremented(),
            FLUSH_BLOCK_INTERVAL - 1,
            FLUSH_TIME_INTERVAL
        ));
        assert!(!is_periodic_flush_due(
            checkpoint_height,
            checkpoint_height,
            FLUSH_BLOCK_INTERVAL,
            FLUSH_TIME_INTERVAL
        ));
        assert!(!is_periodic_flush_due(
            checkpoint_height,
            checkpoint_height.incremented(),
            FLUSH_BLOCK_INTERVAL - 1,
            FLUSH_TIME_INTERVAL - Duration::from_nanos(1)
        ));
        assert!(is_periodic_flush_due(
            checkpoint_height,
            checkpoint_height.incremented(),
            FLUSH_BLOCK_INTERVAL - 1,
            FLUSH_TIME_INTERVAL
        ));
        assert!(is_periodic_flush_due(
            checkpoint_height.incremented(),
            checkpoint_height.incremented().incremented(),
            FLUSH_BLOCK_INTERVAL,
            Duration::ZERO
        ));
    }
}
