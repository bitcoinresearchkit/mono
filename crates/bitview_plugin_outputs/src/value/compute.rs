use std::ops::Range;

use bitview_cohort::{OutputTypeId, SpendableType};
use bitview_compute::prepare_computed;
use bitview_plugin_indexer::{Indexer, overwritten_output};
use bitview_primitives::SupplyState;
use bitview_traversable::Traversable;
use bitview_vecs::CachedSeries;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, Sats};
use statedb::{Amount, Creations};
use vecdb::{AnyStoredVec, AnyVec, Bytes, ReadableVec, VecIndex, WritableVec};

use crate::{STORAGE, by_type::OutputValue, counts::Coinbase};

/// What the walk over each block's outputs fills beside the creation journal.
pub(crate) struct Targets<'a> {
    /// The value sent to OP_RETURN outputs, through every block.
    pub(crate) op_return: &'a mut CachedSeries<Height, Sats>,
    pub(crate) values: &'a mut SpendableType<OutputValue>,
    pub(crate) coinbase: &'a mut Coinbase,
}

impl Targets<'_> {
    fn stored_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut stored: Vec<&mut dyn AnyStoredVec> = vec![&mut *self.op_return];
        stored.extend(
            self.values
                .iter_mut()
                .map(|output| &mut output.value.cumulative.sats.height as &mut dyn AnyStoredVec),
        );
        stored.extend(self.coinbase.iter_any_stored_mut());
        stored
    }
}

pub(crate) fn compute_sats(
    mut targets: Targets<'_>,
    created: &mut Creations,
    range: Range<usize>,
    indexer: &Indexer,
    exit: &Exit,
) -> Result<()> {
    let vecs = indexer.vecs();
    let (first_txout, output_types, values) = (
        &vecs.outputs.first_txout_index,
        &vecs.outputs.output_type,
        &vecs.outputs.value,
    );
    let (first_tx, tx_first_txout, hashes) = (
        &vecs.transactions.first_tx_index,
        &vecs.transactions.first_txout_index,
        &vecs.blocks.blockhash,
    );
    // The producer's own version and its inputs': a bump of either rebuilds the journal.
    let version = STORAGE.schema_version()
        + first_txout.version()
        + output_types.version()
        + values.version()
        + first_tx.version()
        + tx_first_txout.version();
    let end = range.end;
    let start = {
        let _lock = exit.lock();
        created.validate_version(u32::from(version).into())?;
        let start = prepare_computed(
            targets.stored_mut(),
            version,
            range.start.min(created.end()),
            exit,
        )?;
        created.truncate(start)?;
        start
    };
    if start < end {
        // Bounded reads: an update of a few blocks reads only their boundaries and values.
        let mut first_txout = first_txout.bounded_cursor_at(start, end + 1);
        let mut coinbases = first_tx.bounded_cursor_at(start, end);
        let mut output_index = first_txout.next().data()?.to_usize();
        let value_len = values.len();
        let mut values = values.bounded_cursor_at(output_index, value_len);
        let mut output_types = output_types.range_cursor_at(output_index, value_len);
        let mut cumulative = targets.op_return.collect_last().unwrap_or_default();
        let op_return = OutputTypeId::OpReturn.index();
        for h in start..end {
            let next_output = first_txout.next().map_or(value_len, |i| i.to_usize());
            // The coinbase's outputs end where the next transaction's begin.
            let coinbase_outputs = tx_first_txout
                .collect_one(coinbases.next().data()?.incremented())
                .map_or(value_len, |index| index.get().to_usize())
                - output_index;
            let block_outputs = next_output - output_index;
            debug_assert!(coinbase_outputs <= block_outputs);
            // Each output's value and count in its type's slot, OP_RETURN's last: the coinbase's
            // outputs, counted by type too, then the block's others.
            let mut by_type = [(Sats::ZERO, 0u64); OutputTypeId::COUNT];
            let mut add = |slot: usize| {
                let (value, count) = &mut by_type[slot];
                *value += values.next().expect("aligned output value");
                *count += 1;
            };
            let mut coinbase_types = [0u64; OutputTypeId::COUNT];
            output_types.for_each(coinbase_outputs, |output_type| {
                let slot = OutputTypeId::from_output_type(output_type).index();
                coinbase_types[slot] += 1;
                add(slot);
            });
            output_types.for_each(block_outputs - coinbase_outputs, |output_type| {
                add(OutputTypeId::from_output_type(output_type).index())
            });
            output_index = next_output;
            cumulative += by_type[op_return].0;
            targets.op_return.push(cumulative);
            // Every type but OP_RETURN is spendable: the supply created.
            let supply = by_type[..op_return].iter().fold(
                SupplyState::default(),
                |mut supply, &(value, count)| {
                    supply.value += value;
                    supply.utxo_count += count;
                    supply
                },
            );
            for (output, (total, _)) in targets.values.iter_mut().zip(by_type) {
                output.value.push_sats(total);
            }
            targets
                .coinbase
                .push_block(coinbase_outputs as u64, &coinbase_types);
            let supply = if h == 0 {
                SupplyState::default()
            } else {
                supply
            };
            let correction = overwritten_output(Height::from(h)).map(|(origin, v)| {
                (
                    u32::from(origin),
                    Amount {
                        sats: u64::from(v.value),
                        count: v.utxo_count,
                    },
                )
            });
            let _lock = exit.lock();
            created.push(
                hashes.collect_one(Height::from(h)).data()?.to_bytes(),
                Amount {
                    sats: u64::from(supply.value),
                    count: supply.utxo_count,
                },
                correction,
            )?;
        }
    }
    let _lock = exit.lock();
    for vec in targets.stored_mut() {
        vec.write()?;
    }
    created.commit()?;
    Ok(())
}
