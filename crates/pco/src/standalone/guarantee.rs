use crate::data_types::Latent;
use crate::errors::PcoResult;
use crate::standalone::constants::{
  BITS_TO_ENCODE_N_ENTRIES, BITS_TO_ENCODE_STANDALONE_VERSION, BITS_TO_ENCODE_VARINT_POWER,
  MAGIC_HEADER,
};
use crate::wrapped::guarantee as wrapped_guarantee;
use crate::PagingSpec;

/// Returns the maximum possible byte size of a standalone header.
pub fn header_size() -> usize {
  let max_varint_bits = BITS_TO_ENCODE_VARINT_POWER + 64;
  MAGIC_HEADER.len()
    + 1 // uniform dtype
    + (max_varint_bits + BITS_TO_ENCODE_STANDALONE_VERSION).div_ceil(8) as usize
    + wrapped_guarantee::header_size()
}

/// Returns the maximum possible byte size of a standalone chunk for a given
/// latent type (e.g. u32 or u64) and count of numbers.
pub fn chunk_size<L: Latent>(n: usize) -> usize {
  1 + BITS_TO_ENCODE_N_ENTRIES.div_ceil(8) as usize + wrapped_guarantee::chunk_size::<L>(n)
}

/// Returns the maximum possible byte size of a standalone file given a
/// latent type (e.g. u32 or u64), count of numbers, and `PagingSpec`.
///
/// Will return an invalid argument error if the paging spec is invalid.
pub fn file_size<L: Latent>(n: usize, paging_spec: &PagingSpec) -> PcoResult<usize> {
  let n_per_chunk = paging_spec.n_per_page(n)?;
  let res = header_size()
    + n_per_chunk
      .iter()
      .map(|&chunk_n| chunk_size::<L>(chunk_n))
      .sum::<usize>()
    + 1;
  Ok(res)
}
