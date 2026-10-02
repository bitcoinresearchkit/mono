use std::ops::Range;

use crate::overwritten_output;
use bitview_compute::prepare_computed;
use bitview_vecs::CachedSeries;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{BlockHash, Height, OutputType, Sats, SupplyState, TxOutIndex};
use statedb::{Amount, Creations};
use vecdb::{
    AnyStoredVec, AnyVec, Bytes, BytesVec, OverflowVec, ReadableVec, VecIndex, WritableVec,
};

pub(crate) fn compute_sats(
    target: &mut CachedSeries<Height, Sats>,
    created: &mut Creations,
    range: Range<usize>,
    first_txout: &impl ReadableVec<Height, TxOutIndex>,
    output_data: (
        &BytesVec<TxOutIndex, OutputType>,
        &OverflowVec<TxOutIndex, Sats>,
    ),
    hashes: &impl ReadableVec<Height, BlockHash>,
    exit: &Exit,
) -> Result<()> {
    let (output_types, values) = output_data;
    let version = first_txout.version() + output_types.version() + values.version();
    let end = range.end.min(first_txout.len());
    let start = {
        let _lock = exit.lock();
        created.validate_version(u32::from(version).into())?;
        let start = prepare_computed(
            [target as &mut dyn AnyStoredVec],
            version,
            range.start.min(end).min(created.end()),
            exit,
        )?;
        created.truncate(start)?;
        start
    };
    if start < end {
        let mut first_txout = first_txout.cursor();
        first_txout.advance(start);
        let mut output_index = first_txout.next().data()?.to_usize();
        let value_len = values.len();
        let mut values = values.cursor();
        values.advance(output_index);
        let mut output_types = output_types.range_cursor_at(output_index, value_len);
        let mut cumulative = target.collect_last().unwrap_or_default();
        for h in start..end {
            let mut supply = SupplyState::default();
            let next_output = first_txout.next().map_or(value_len, |i| i.to_usize());
            output_types.for_each(next_output - output_index, |output_type| {
                let value = values.next().expect("aligned output value");
                if output_type == OutputType::OpReturn {
                    cumulative += value;
                } else {
                    supply.value += value;
                    supply.utxo_count += 1;
                }
                output_index += 1;
            });
            target.push(cumulative);
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
    target.write()?;
    created.commit()?;
    Ok(())
}
