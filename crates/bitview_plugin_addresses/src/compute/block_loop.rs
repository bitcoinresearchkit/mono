use std::ops::{ControlFlow, Range};

use bitview_cohort::AddressTypeId;
use bitview_distribution::readers::{BatchColumns, BlockBounds, index_range};
use bitview_plugin_indexer::{Indexer, overwritten_output};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{ByAddrType, FundedAddressData, TypeIndex};
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Cents, Height, OutputType, Sats, TxIndex};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::{debug, info};
use vecdb::{AnyVec, ReadableVec, Stamp, VecIndex, unlikely};

use super::{AddrReaders, Workspace, readers::tx_indexes};
use crate::{
    Vecs,
    addr::{MemberState, SHARDS, SourcedAddrData},
    block::{
        AddrTypeLookup, Received, TransferAddressCache, TxIndexes, process_inputs, process_lost,
        process_outputs, process_received, process_sent,
    },
    compute::write::write,
    state::{AddrStates, CohortLog},
};

/// A BIP30-overwritten coinbase output, removed from its P2PK65 address without a spend.
#[derive(Clone, Copy)]
struct Lost {
    height: usize,
    type_index: TypeIndex,
    value: Sats,
    creation_price: Cents,
}

/// One address type's activity in one block, for one shard of its addresses.
struct TypeBlock {
    received: FxHashMap<TypeIndex, Received>,
    /// Spends in input order, with their creation prices.
    spends: Vec<(TypeIndex, Sats, Cents)>,
    /// Transactions each address spent in.
    sent_txs: FxHashMap<TypeIndex, TxIndexes>,
}

/// Flush the address cache early once it holds this many addresses (checked after each batch):
/// it bounds the cache's tables (about 57 bytes a slot) however busy the chain gets. Run alone to
/// ~970,750 blocks (16 GB Mac mini, external SSD), the plugin takes 556 s and peaks at 8.4 GiB;
/// 32M takes 552 s and 9.8 GiB, 16M 632 s and 7.2 GiB.
const MAX_CACHED_ADDRS: usize = 24_000_000;

/// Process `blocks` until the address cache is full, flush, and return the next height. The
/// protected `tail` is written with its undo, and its address state stays staged until the
/// update's publication: at most a few blocks' changes, while the bulk of a catch-up writes its
/// state at once (address stats answer 503 until such an update publishes).
#[allow(clippy::too_many_arguments)]
pub fn process_chunk(
    vecs: &mut Vecs,
    addr_states: &mut AddrStates,
    indexer: &Indexer,
    mappings: &MappingsVecs,
    workspace: &mut Workspace<'_>,
    blocks: Range<usize>,
    prices: &[Cents],
    tail: bool,
    exit: &Exit,
) -> Result<usize> {
    let start_usize = blocks.start;
    let end_usize = blocks.end;

    let height_to_first_tx_index = &indexer.vecs().transactions.first_tx_index;
    let tx_index_to_output_count = &mappings.tx_index.output_count;
    let tx_index_to_input_count = &mappings.tx_index.input_count;

    let height_to_first_tx_index_vec: Vec<TxIndex> = height_to_first_tx_index.collect_range_at(
        start_usize,
        (end_usize + 1).min(height_to_first_tx_index.len()),
    );

    let lost = blocks
        .clone()
        .filter_map(|height| {
            let (origin, supply) = overwritten_output(Height::from(height))?;
            Some((height, origin, supply.value))
        })
        .map(|(height, origin, value)| -> Result<Lost> {
            let vecs = indexer.vecs();
            let coinbase = vecs
                .transactions
                .first_tx_index
                .collect_one(origin)
                .data()?;
            let output = vecs
                .transactions
                .first_txout_index
                .collect_one(coinbase)
                .data()?
                .get();
            debug_assert_eq!(
                vecs.outputs.output_type.collect_one(output),
                Some(OutputType::P2PK65)
            );
            Ok(Lost {
                height,
                type_index: vecs.outputs.type_index.collect_one(output).data()?,
                value,
                creation_price: prices[usize::from(origin)],
            })
        })
        .collect::<Result<Vec<_>>>()?;

    debug!("creating AddrReaders");
    let vr = AddrReaders::new(&vecs.state);
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
            &vecs.state,
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
                    prices,
                );
                (received, inputs.spends, inputs.tx_index_vecs)
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
        let mut base = addr_states.metrics.clone();
        base.reset_per_block();
        let base = &base;
        let processed = cache
            .shards_mut()
            .zip(by_shard.into_iter().flat_map(ByAddrType::into_iter))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|((output_type, addrs), (blocks_type, blocks))| {
                debug_assert_eq!(output_type, blocks_type);
                // P2A is not an address type: its addresses update only their own data.
                let id = AddressTypeId::from_output_type(output_type);
                let member = id.map(|id| *id.select(&base.0));
                let first = batch.blocks.start;
                let blocks = process_shard(
                    output_type,
                    addrs,
                    blocks,
                    first,
                    block_prices,
                    member,
                    &lost,
                );
                (id, blocks)
            })
            .collect::<Vec<_>>();

        for (offset, height) in batch.blocks.clone().enumerate() {
            let height = Height::from(height);
            if unlikely(height.is_multiple_of(100)) {
                info!("Computing metrics at block {height}...");
            } else {
                debug!("Processing chain at {height}...");
            }
            let metrics = &mut addr_states.metrics;
            metrics.clone_from(base);
            for (id, blocks) in &processed {
                if let (Some(id), (cohorts, Some(member))) = (*id, &blocks[offset]) {
                    cohorts.apply_to(&mut addr_states.amount_range);
                    metrics.add_member_delta(id, *member, base);
                }
            }
            vecs.all.push(&metrics.all());
            for (member, member_state) in vecs.types.iter_mut().zip(metrics.0.iter()) {
                member.push(member_state);
            }
            vecs.balances
                .push(&addr_states.amount_range, block_prices[offset]);
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
    // A failure before the state is written leaves its changed buffers for the next update to discard.
    vecs.unpublished_state = Some((Stamp::from(last_height), tail));
    cache.flush_into(&mut vecs.state)?;
    let finished = tail && next == end_usize;
    // Every write of the tail keeps its changes, so a reorg can roll back across early flushes.
    write(vecs, addr_states, last_height, tail, finished)?;
    if !finished {
        vecs.flush()?;
    }

    Ok(next)
}

/// Apply one shard of an address type's blocks in order: per block its transaction counts,
/// receives, then spends (the block-level convention the README states). `member` is the type's
/// counters (none for P2A) the shard starts from. Returns each block's cohort changes and the
/// member after it.
#[allow(clippy::too_many_arguments)]
fn process_shard(
    output_type: OutputType,
    addrs: &mut FxHashMap<TypeIndex, SourcedAddrData<FundedAddressData>>,
    blocks: Vec<TypeBlock>,
    first_height: usize,
    prices: &[Cents],
    mut member: Option<MemberState>,
    lost: &[Lost],
) -> Vec<(CohortLog, Option<MemberState>)> {
    let lost = lost
        .iter()
        .filter(|lost| output_type == OutputType::P2PK65 && addrs.contains_key(&lost.type_index))
        .copied()
        .collect::<Vec<_>>();
    let mut lookup = AddrTypeLookup::new(addrs);
    let mut transfers = TransferAddressCache::default();
    let mut processed = Vec::with_capacity(blocks.len());
    for (
        height,
        (
            TypeBlock {
                received,
                spends,
                sent_txs,
            },
            &price,
        ),
    ) in (first_height..).zip(blocks.into_iter().zip(prices))
    {
        if let Some(member) = &mut member {
            member.reset_per_block();
        }
        lookup.update_tx_counts(&received, sent_txs);
        transfers.prepare(received.keys().copied());
        let mut cohorts = CohortLog::default();
        process_received(
            received,
            &mut cohorts,
            &mut lookup,
            price,
            output_type,
            member.as_mut(),
        );
        process_sent(
            &spends,
            &mut cohorts,
            &mut lookup,
            price,
            output_type,
            member.as_mut(),
            &mut transfers,
        );
        for lost in lost.iter().filter(|lost| lost.height == height) {
            process_lost(
                lost.type_index,
                lost.value,
                lost.creation_price,
                &mut cohorts,
                &mut lookup,
                output_type,
                member.as_mut(),
            );
        }
        processed.push((cohorts, member));
    }
    processed
}
