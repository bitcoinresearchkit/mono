use brk_types::{CpfpClusterTxIndex, Sats, VSize};

/// One cluster member's input to Single Fee Linearization: its
/// `(fee, vsize)` and parent edges as local indices into the same array.
pub struct ChunkInput<'a> {
    pub fee: Sats,
    pub vsize: VSize,
    pub parents: &'a [CpfpClusterTxIndex],
}
