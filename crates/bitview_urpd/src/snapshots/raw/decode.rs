use brk_error::{Error, Result};
use pco::{
    data_types::Number,
    standalone::{DecompressorItem, FileDecompressor},
};

/// Decode exactly the declared count and section, without trusting PCO's size
/// hint. Check each chunk's count before allocating its output buffer.
pub fn exact<T: Number>(mut bytes: &[u8], expected: usize) -> Result<Vec<T>> {
    let (file, rest) = FileDecompressor::new(bytes)?;
    bytes = rest;
    let mut values = Vec::new();
    loop {
        match file.chunk_decompressor(bytes)? {
            DecompressorItem::Chunk(mut chunk) => {
                let count = chunk.n();
                if count > expected - values.len() {
                    return Err(Error::Deserialization("UrpdRaw: too many entries".into()));
                }
                let start = values.len();
                values.try_reserve_exact(count).map_err(|_| {
                    Error::Deserialization("UrpdRaw: cannot allocate entries".into())
                })?;
                values.resize(start + count, T::default());
                let progress = chunk.read(&mut values[start..])?;
                if !progress.finished || progress.n_processed != count {
                    return Err(Error::Deserialization("UrpdRaw: incomplete chunk".into()));
                }
                bytes = chunk.into_src();
            }
            DecompressorItem::EndOfData(rest) => {
                if values.len() != expected || !rest.is_empty() {
                    return Err(Error::Deserialization(
                        "UrpdRaw: entry count or compressed section length mismatch".into(),
                    ));
                }
                return Ok(values);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/snapshots/decode.rs"]
mod tests;
