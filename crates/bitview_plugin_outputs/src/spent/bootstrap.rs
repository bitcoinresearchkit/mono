use std::{
    io::{BufWriter, Read, Result as IoResult, Seek, Write},
    iter::repeat_n,
};

use bitview_primitives::{Index40, TxInIndex, TxOutIndex};
use brk_error::Result;
use brk_exit::Exit;
use tempfile::tempfile_in;
use tracing::info;
use vecdb::{AnyStoredVec, AnyVec, Error as VecError, ReadableVec, Stamp, VecIndex, WritableVec};

use super::Vecs;

// Bound the working output range to 320 MiB, independent of chain length.
pub(super) const RANGE_LEN: usize = 1 << 26;
const RECORD_LEN: usize = size_of::<u64>();

pub(super) fn reset_incomplete(vecs: &mut Vecs) -> Result<()> {
    // A partial output range is not a block-level resume point.
    if vecs.txin_index.stamp() == Stamp::default() && !vecs.txin_index.is_empty() {
        vecs.txin_index.reset()?;
    }
    Ok(())
}

/// Build an empty inverse index in output order. Scratch files are unlinked
/// while open, so cancellation and process termination leave nothing to clean up.
/// Each spend needs one 8-byte scratch record, freed as its range is consumed.
/// The caller publishes a nonzero stamp only after the entire baseline is durable.
pub(super) fn build(
    vecs: &mut Vecs,
    inputs: &impl ReadableVec<TxInIndex, TxOutIndex>,
    input_end: usize,
    output_end: usize,
    exit: &Exit,
) -> Result<()> {
    build_ranges(vecs, inputs, input_end, output_end, RANGE_LEN, exit)
}

fn build_ranges(
    vecs: &mut Vecs,
    inputs: &impl ReadableVec<TxInIndex, TxOutIndex>,
    input_end: usize,
    output_end: usize,
    max_range_len: usize,
    exit: &Exit,
) -> Result<()> {
    // Persist the empty, unstamped state before replacing any stored ranges,
    // including when rebuilding an existing index after a reset.
    {
        let _lock = exit.lock();
        vecs.txin_index.flush()?;
    }
    let range_bits = range_bits(input_end, max_range_len);
    let range_len = 1_usize << range_bits;
    let mask = range_len as u64 - 1;
    let path = vecs.txin_index.db_path();
    let mut ranges = (0..output_end.div_ceil(range_len))
        .map(|_| tempfile_in(&path).map(|file| BufWriter::with_capacity(1 << 20, file)))
        .collect::<IoResult<Vec<_>>>()?;

    info!("Partitioning spent outputs...");
    let mut input = 0_u64;
    inputs.try_for_each_range_at(0, input_end, |output| -> Result<()> {
        if !output.is_coinbase() {
            let index = output.to_usize();
            if index >= output_end {
                return Err(VecError::IndexTooHigh {
                    index,
                    len: output_end,
                    name: inputs.name().to_owned(),
                }
                .into());
            }
            let record = (input << range_bits) | (index as u64 & mask);
            ranges[index >> range_bits].write_all(&record.to_le_bytes())?;
        }
        input += 1;
        Ok(())
    })?;

    let mut records = vec![0; RECORD_LEN * 65_536];
    for (range, file) in ranges.into_iter().enumerate() {
        let mut file = file.into_inner().map_err(|error| error.into_error())?;
        file.rewind()?;
        let mut remaining = file.metadata()?.len() as usize;
        let len = range_len.min(output_end - range * range_len);
        vecs.txin_index
            .extend(repeat_n(Index40::new(TxInIndex::UNSPENT), len));
        let values = vecs.txin_index.pushed_mut();

        // Records retain input order, including last-write-wins for duplicates.
        while remaining > 0 {
            let len = remaining.min(records.len());
            file.read_exact(&mut records[..len])?;
            for record in records[..len].as_chunks::<RECORD_LEN>().0 {
                let record = u64::from_le_bytes(*record);
                values[(record & mask) as usize] =
                    Index40::new(TxInIndex::new(record >> range_bits));
            }
            remaining -= len;
        }

        let _lock = exit.lock();
        vecs.txin_index.flush()?;
        info!(
            "Building spent outputs: {:.0}%",
            (range * range_len + len) as f64 / output_end as f64 * 100.0
        );
    }
    Ok(())
}

// Pack the input index and the offset within its output range into one u64.
// Shrink ranges as input indexes grow so packing never truncates an input index.
fn range_bits(input_end: usize, max_range_len: usize) -> u32 {
    max_range_len
        .ilog2()
        .min((input_end.saturating_sub(1) as u64).leading_zeros())
}
