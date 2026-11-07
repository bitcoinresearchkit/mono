use std::{
    mem,
    ops::{ControlFlow, Range},
};

use bitview_cohort::{AddressType, AddressTypeId, AmountRange};
use bitview_distribution::readers::{BatchColumns, BlockBounds, index_range};
use bitview_plugin_indexer::{Indexer, overwritten_output};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{AddrTypeId, ByAddrType, TypeIndex};
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Cents, Height, OutputType, Sats, TxIndex};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tracing::info;
use vecdb::{AnyVec, ReadableVec, Stamp, VecIndex, unlikely};

use super::{AddrReaders, Workspace, readers::tx_indexes};
use crate::{
    Vecs,
    addr::{AddrMetricsState, AddressVecs, Changed, MemberState, SHARDS, SourcedAddrData},
    balance::Balances,
    block::{
        AddrTypeLookup, ShardBlocks, TransferAddressCache, TypeBlock, process_inputs, process_lost,
        process_outputs, process_received, process_sent,
    },
    compute::write::{UnpublishedState, write},
    state::{AddrCohortState, AddrStates, CohortLog},
};

/// A batch's per-block series values and band updates, deferred to run beside the next batch's
/// loading and grouping, which read neither the bands nor the series.
struct PendingApply {
    first_height: usize,
    /// The counters the batch's shards started from.
    base: AddrMetricsState,
    ids: Vec<Option<AddressTypeId>>,
    processed: Vec<(CohortLog, Option<MemberState>)>,
    block_count: usize,
}

impl PendingApply {
    /// Each block in order: its cohort changes into the bands, then every series' value.
    fn apply(
        self,
        all: &mut AddressVecs,
        types: &mut AddressType<AddressVecs>,
        balances: &mut Balances,
        amount_range: &mut AmountRange<AddrCohortState>,
        prices: &[Cents],
    ) {
        let Self {
            first_height,
            base,
            ids,
            processed,
            block_count,
        } = self;
        let mut metrics = base.clone();
        for offset in 0..block_count {
            let height = first_height + offset;
            if unlikely(height.is_multiple_of(100)) {
                info!("Computing address metrics at block {height}...");
            }
            metrics.clone_from(&base);
            for (id, blocks) in ids.iter().zip(processed.chunks(block_count)) {
                if let (Some(id), (cohorts, Some(member))) = (*id, &blocks[offset]) {
                    cohorts.apply_to(amount_range);
                    metrics.add_member_delta(id, *member, &base);
                }
            }
            all.push(&metrics.all());
            for (member, member_state) in types.iter_mut().zip(metrics.0.iter()) {
                member.push(member_state);
            }
            balances.push(amount_range, prices[height]);
            amount_range
                .iter_mut()
                .for_each(|state| state.inner.reset_block());
        }
    }
}

/// A BIP30-overwritten coinbase output, removed from its P2PK65 address without a spend.
struct Lost {
    height: usize,
    type_index: TypeIndex,
    value: Sats,
    creation_price: Cents,
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

    let state_readers = AddrReaders::new(&vecs.state);

    let Workspace {
        columns,
        addresses: cache,
    } = workspace;
    let tx_count = indexer.vecs().transactions.txid.len();

    // Each type's first address index per block of the range, for telling new addresses apart.
    let addrs = &indexer.vecs().addrs;
    macro_rules! first_indexes {
        ($(($variant:ident, $field:ident, $length:ident, $index:path, $bytes:path)),+ $(,)?) => {
            ByAddrType {
                $($field: addrs
                    .$field
                    .first_index
                    .collect_range_at(start_usize, end_usize)
                    .into_iter()
                    .map(|index| TypeIndex::from(index.to_usize()))
                    .collect::<Vec<_>>(),)+
            }
        };
    }
    let first_indexes = bitview_primitives::with_address_storage_types!(first_indexes);

    // Load cold address state once for each batch of blocks.
    let bounds = BlockBounds::new(indexer, blocks);
    let mut next = end_usize;
    let mut pending: Option<PendingApply> = None;
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
        let first_addr_indexes = ByAddrType::from_fn(|id| id.select(&first_indexes)[offset]);

        let previous = pending.take();
        let (all, types, balances) = (&mut vecs.all, &mut vecs.types, &mut vecs.balances);
        let amount_range = &mut addr_states.amount_range;
        let state = &vecs.state;
        let (_, lanes) = rayon::join(
            || {
                if let Some(previous) = previous {
                    previous.apply(all, types, balances, amount_range, prices);
                }
            },
            || {
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
                    &state_readers,
                    state,
                );

                // Group each block's outputs and inputs by address, blocks in parallel.
                let groups = batch
                    .blocks
                    .clone()
                    .into_par_iter()
                    .map(|height| {
                        let offset = height - start_usize;
                        let transactions = index_range(
                            &height_to_first_tx_index_vec,
                            offset,
                            offset + 1,
                            tx_count,
                        );
                        let first_tx = TxIndex::from(transactions.start);
                        let tx_len = transactions.len() as u64;
                        let output_range = batch.block_outputs(height);
                        // Omit this block's coinbase, including coinbase-only blocks.
                        let input_range = batch.block_spends(height);
                        let mut blocks = ShardBlocks::default();
                        process_outputs(
                            &mut blocks,
                            tx_indexes(first_tx, tx_len, tx_index_to_output_count),
                            &output_values[output_range.clone()],
                            &output_types[output_range.clone()],
                            &output_indexes[output_range],
                        );
                        process_inputs(
                            &mut blocks,
                            tx_indexes(first_tx, tx_len, tx_index_to_input_count).skip(1),
                            &input_values[input_range.clone()],
                            &input_types[input_range.clone()],
                            &input_indexes[input_range.clone()],
                            &input_heights[input_range],
                            prices,
                        );
                        blocks
                    })
                    .collect::<Vec<_>>();
                // Every shard of every type is a lane holding its blocks side by side, lanes in
                // `shards_mut` order.
                let block_count = batch.blocks.len();
                let mut lanes = Vec::new();
                lanes.resize_with(
                    SHARDS * AddrTypeId::ALL.len() * block_count,
                    TypeBlock::default,
                );
                for (offset, blocks) in groups.into_iter().enumerate() {
                    let blocks = blocks.into_iter().flat_map(|shard| shard.into_iter());
                    for (lane, (_, block)) in blocks.enumerate() {
                        lanes[lane * block_count + offset] = block;
                    }
                }
                lanes
            },
        );
        let mut lanes = lanes;
        let block_count = batch.blocks.len();

        // Each address touches only its own state, so every shard of every type runs apart from
        // the same base metrics; cohort changes are logged per block and applied below in order.
        let block_prices = &prices[batch.blocks.clone()];
        let mut base = addr_states.metrics.clone();
        base.reset_per_block();
        let base = &base;
        let mut processed = Vec::new();
        processed.resize_with(lanes.len(), Default::default);
        let ids = cache
            .shards_mut()
            .zip(lanes.chunks_mut(block_count))
            .zip(processed.chunks_mut(block_count))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(((output_type, addrs), blocks), processed)| {
                // P2A is not an address type: its addresses update only their own data.
                let id = AddressTypeId::from_output_type(output_type);
                let member = id.map(|id| *id.select(&base.0));
                process_shard(
                    output_type,
                    addrs,
                    blocks,
                    batch.blocks.start,
                    block_prices,
                    member,
                    &lost,
                    processed,
                );
                id
            })
            .collect::<Vec<_>>();

        // The next batch's shards start from this one's last counters: set now, from each lane's
        // state after its last block; the per-block values and bands follow beside it.
        let mut last = base.clone();
        for (id, blocks) in ids.iter().zip(processed.chunks(block_count)) {
            if let (Some(id), (_, Some(member))) = (*id, &blocks[block_count - 1]) {
                last.add_member_delta(id, *member, base);
            }
        }
        addr_states.metrics = last;
        pending = Some(PendingApply {
            first_height: batch.blocks.start,
            base: base.clone(),
            ids,
            processed,
            block_count,
        });
        if cache.len() >= MAX_CACHED_ADDRS {
            next = batch.blocks.end;
            return Ok(ControlFlow::Break(()));
        }
        Ok(ControlFlow::Continue(()))
    })?;
    if let Some(pending) = pending {
        pending.apply(
            &mut vecs.all,
            &mut vecs.types,
            &mut vecs.balances,
            &mut addr_states.amount_range,
            prices,
        );
    }
    let finished = tail && next == end_usize;
    // A write outside the update's publication holds readers to the current one, rather than
    // excluding them for its duration.
    let held = if finished {
        None
    } else {
        cache.held(indexer.pin().lengths().height, &state_readers, &vecs.state)
    };
    drop(state_readers);
    let last_height = Height::from(next - 1);
    let _lock = exit.lock();
    // A failure before the state is written leaves its changed buffers for the next update to discard.
    let unpublished = vecs.unpublished_state.insert(UnpublishedState {
        stamp: Stamp::from(last_height),
        with_changes: tail,
        changed: finished.then(Changed::new),
        held,
        published: false,
    });
    cache.flush_into(&mut vecs.state, unpublished.changed.as_mut())?;
    // Every write of the tail keeps its changes, so a reorg can roll back across early flushes.
    write(vecs, addr_states, last_height, tail, finished)?;
    if !finished {
        vecs.db.flush();
    }

    Ok(next)
}

/// Apply one shard of an address type's blocks in order: per block its transaction counts,
/// receives, then spends (the block-level convention the README states). `member` is the type's
/// counters (none for P2A) the shard starts from. Writes each block's cohort changes and the
/// member after it to `processed`.
#[allow(clippy::too_many_arguments)]
fn process_shard(
    output_type: OutputType,
    addrs: &mut FxHashMap<TypeIndex, SourcedAddrData>,
    blocks: &mut [TypeBlock],
    first_height: usize,
    prices: &[Cents],
    mut member: Option<MemberState>,
    lost: &[Lost],
    processed: &mut [(CohortLog, Option<MemberState>)],
) {
    let lost = if output_type == OutputType::P2PK65 {
        lost
    } else {
        &[]
    };
    let mut lookup = AddrTypeLookup::new(addrs);
    let mut transfers = TransferAddressCache::default();
    for (height, ((block, &price), processed)) in
        (first_height..).zip(blocks.iter_mut().zip(prices).zip(processed))
    {
        let TypeBlock {
            received,
            spends,
            sent_txs,
        } = mem::take(block);
        if let Some(member) = &mut member {
            member.reset_per_block();
        }
        lookup.update_tx_counts(&received, sent_txs);
        transfers.prepare(received.keys().copied());
        let mut cohorts = CohortLog::with_capacity(received.len() + spends.len());
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
        // The duplicate coinbase pays the overwritten output's address in this block, so by now
        // the address is in this shard's map exactly when the shard owns it.
        for lost in lost.iter().filter(|lost| lost.height == height) {
            if !lookup.contains(lost.type_index) {
                continue;
            }
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
        *processed = (cohorts, member);
    }
}
