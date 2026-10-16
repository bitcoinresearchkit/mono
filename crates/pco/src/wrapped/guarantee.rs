use crate::data_types::Latent;
use crate::metadata::chunk_latent_var::ChunkLatentVarMeta;
use crate::metadata::per_latent_var::PerLatentVar;
use crate::metadata::{Bin, ChunkMeta, DeltaEncoding, DynBins, FormatVersion, Mode};

/// Returns the maximum possible byte size of a wrapped header.
pub fn header_size() -> usize {
  FormatVersion::MAX_ENCODED_SIZE
}

pub(crate) fn baseline_chunk_meta<L: Latent>() -> ChunkMeta {
  let primary = ChunkLatentVarMeta {
    ans_size_log: 0,
    bins: DynBins::new(vec![Bin {
      weight: 1,
      lower: L::ZERO,
      offset_bits: L::BITS,
    }]),
  };

  ChunkMeta::new(
    Mode::Classic,
    DeltaEncoding::NoOp,
    PerLatentVar {
      delta: None,
      primary,
      secondary: None,
    },
  )
  .unwrap()
}

/// Returns the maximum possible byte size of a wrapped chunk for a given
/// latent type (e.g. u32 or u64) and count of numbers.
pub(crate) fn chunk_size<L: Latent>(n: usize) -> usize {
  baseline_chunk_meta::<L>().max_size() + (n * L::BITS as usize).div_ceil(8)
}
