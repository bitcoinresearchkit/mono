use crate::util::invalid;
use pco::{
    ChunkConfig, DeltaSpec, ModeSpec,
    data_types::Number,
    standalone::{simple_compress, simple_decompress_into},
};
use std::io::Result;

/// The PCO settings of every packed column: spend rows, cell records and retained snapshots (these
/// at `snapshot::PACKED_LEVEL`).
pub(crate) fn config() -> ChunkConfig {
    ChunkConfig::default()
        .with_compression_level(3)
        .with_mode_spec(ModeSpec::Classic)
        .with_delta_spec(DeltaSpec::NoOp)
}

/// Appends `column` as one length-prefixed PCO stream.
pub(crate) fn put<T: Number>(out: &mut Vec<u8>, column: &[T]) -> Result<()> {
    let bytes = simple_compress(column, &config()).map_err(|_| invalid("column compression failed"))?;
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&bytes);
    Ok(())
}

/// Decodes the stream at `*at` into all of `column`, moving `at` past it.
pub(crate) fn take<T: Number>(bytes: &[u8], at: &mut usize, column: &mut [T]) -> Result<()> {
    let len = word(bytes, *at)? as usize;
    let stream = bytes
        .get(*at + 4..*at + 4 + len)
        .ok_or_else(|| invalid("truncated column"))?;
    let progress = simple_decompress_into(stream, column).map_err(|_| invalid("corrupt column"))?;
    if progress.n_processed != column.len() || !progress.finished {
        return Err(invalid("column length mismatch"));
    }
    *at += 4 + len;
    Ok(())
}

/// The little-endian u32 at `at`.
pub(crate) fn word(bytes: &[u8], at: usize) -> Result<u32> {
    let word = bytes.get(at..at + 4).ok_or_else(|| invalid("truncated record"))?;
    Ok(u32::from_le_bytes(word.try_into().unwrap()))
}

/// Packs strictly ascending `keys` as their gaps (the first from zero), the form their stream
/// packs best; `keys` holds the gaps after.
pub(crate) fn put_keys(out: &mut Vec<u8>, keys: &mut [u32]) -> Result<()> {
    let mut previous = None;
    for key in keys.iter_mut() {
        if previous.is_some_and(|previous| *key <= previous) {
            return Err(invalid("keys not strictly ascending"));
        }
        let gap = *key - previous.unwrap_or(0);
        previous = Some(*key);
        *key = gap;
    }
    put(out, keys)
}

/// The keys `put_keys` packed, back from their gaps.
pub(crate) fn take_keys(bytes: &[u8], at: &mut usize, keys: &mut [u32]) -> Result<()> {
    take(bytes, at, keys)?;
    let mut key = 0u32;
    for gap in keys {
        key = key.checked_add(*gap).ok_or_else(|| invalid("key overflow"))?;
        *gap = key;
    }
    Ok(())
}
