use brk_types::{Bytes, Sats, VSize};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BlockMetrics {
    pub(crate) output_count: u64,
    pub(crate) data_bytes: Bytes,
    pub(crate) tx_count: u64,
    pub(crate) tx_vsize: VSize,
    pub(crate) fees: Sats,
}
