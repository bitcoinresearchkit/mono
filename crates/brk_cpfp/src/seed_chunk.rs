use brk_types::{CpfpClusterChunk, CpfpClusterTxIndex, FeeRate};

/// Find the chunk containing `seed_local` and return `(chunk_index,
/// feerate)`. Falls back to `(0, fallback)` when the seed isn't in any
/// chunk - shouldn't happen for a well-formed linearization but keeps
/// callers' wire shape valid.
pub fn find_seed_chunk(
    chunks: &[CpfpClusterChunk],
    seed_local: CpfpClusterTxIndex,
    fallback: FeeRate,
) -> (u32, FeeRate) {
    chunks
        .iter()
        .enumerate()
        .find(|(_, ch)| ch.txs.contains(&seed_local))
        .map(|(i, ch)| (i as u32, ch.feerate))
        .unwrap_or((0, fallback))
}
