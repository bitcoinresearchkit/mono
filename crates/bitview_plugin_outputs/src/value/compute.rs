use bitview_compute::prepare_computed;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_vecs::CachedSeries;
use brk_error::{OptionData, Result};
use brk_exit::Exit;
use brk_types::{Height, OutputType, Sats, TxOutIndex};
use vecdb::{AnyStoredVec, AnyVec, BytesVec, OverflowVec, ReadableVec, VecIndex, WritableVec};

use super::Vecs;

pub fn compute(vecs: &mut Vecs, indexer: &Indexer, prices: &PriceVecs, exit: &Exit) -> Result<()> {
    let outputs = &indexer.vecs().outputs;
    let max_from = indexer.safe_lengths().height;
    compute_sats(
        &mut vecs.op_return.cumulative.sats.height,
        max_from,
        &outputs.first_txout_index,
        &outputs.output_type,
        &outputs.value,
        exit,
    )?;
    vecs.op_return
        .compute_cents(max_from, &prices.spot.cents.height, exit)
}

pub(super) fn compute_sats(
    target: &mut CachedSeries<Height, Sats>,
    max_from: Height,
    first_txout: &impl ReadableVec<Height, TxOutIndex>,
    output_types: &BytesVec<TxOutIndex, OutputType>,
    values: &OverflowVec<TxOutIndex, Sats>,
    exit: &Exit,
) -> Result<()> {
    let version = first_txout.version() + output_types.version() + values.version();
    let end = first_txout.len();
    let start = prepare_computed(
        [target as &mut dyn AnyStoredVec],
        version,
        usize::from(max_from).min(end),
    )?;
    if start < end {
        let mut first_txout = first_txout.cursor();
        first_txout.advance(start);
        let mut output_index = first_txout.next().data()?.to_usize();
        let values = values.reader();
        let mut output_types = output_types.range_cursor_at(output_index, values.len());
        let mut cumulative = target.collect_last().unwrap_or_default();
        for _ in start..end {
            let next_output = first_txout.next().map_or(values.len(), |i| i.to_usize());
            output_types.for_each(next_output - output_index, |output_type| {
                if output_type == OutputType::OpReturn {
                    cumulative += values.get_at(output_index);
                }
                output_index += 1;
            });
            target.push(cumulative);
        }
    }
    let _lock = exit.lock();
    target.write()?;
    Ok(())
}
